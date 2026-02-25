use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::math::{ColorRgb, Direction3, Point3};

use super::{Camera, Lighting, Material, Scene, SceneValidationError, Sphere};

#[derive(Debug, Error)]
pub enum SceneParseError {
    #[error("failed to read scene file '{path}': {source}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("line {line}: unknown directive '{directive}'")]
    UnknownDirective { line: usize, directive: String },

    #[error("line {line}: global field '{field}' is not valid inside a sphere block")]
    GlobalFieldInsideSphere { line: usize, field: &'static str },

    #[error("line {line}: 'Sphere' does not take any values")]
    SphereDirectiveHasValues { line: usize },

    #[error("line {line}: sphere field '{field}' appears before any 'Sphere' directive")]
    SphereFieldBeforeSphere { line: usize, field: &'static str },

    #[error("line {line}: duplicate field '{field}' (first defined on line {first_defined_line})")]
    DuplicateField {
        line: usize,
        field: &'static str,
        first_defined_line: usize,
    },

    #[error("line {line}: field '{field}' expects {expected} value(s), found {found}")]
    WrongValueCount {
        line: usize,
        field: &'static str,
        expected: usize,
        found: usize,
    },

    #[error("line {line}: field '{field}' has invalid number '{token}'")]
    InvalidNumber {
        line: usize,
        field: &'static str,
        token: String,
    },

    #[error("line {line}: missing required field '{field}'")]
    MissingRequiredField { line: usize, field: &'static str },

    #[error("line {line}: {source}")]
    ValidationAtLine {
        line: usize,
        #[source]
        source: SceneValidationError,
    },

    #[error("lines {left_line} and {right_line}: {source}")]
    ValidationAcrossLines {
        left_line: usize,
        right_line: usize,
        #[source]
        source: SceneValidationError,
    },
}

pub fn parse_scene_file(path: impl AsRef<Path>) -> Result<Scene, SceneParseError> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).map_err(|source| SceneParseError::ReadFile {
        path: path.display().to_string(),
        source,
    })?;

    parse_scene_text(&contents)
}

pub fn parse_scene_text(contents: &str) -> Result<Scene, SceneParseError> {
    let mut parser = SceneTextParser::new();
    for (line_index, raw_line) in contents.lines().enumerate() {
        let line_number = line_index + 1;
        parser.parse_line(line_number, raw_line)?;
    }

    parser.finish()
}

#[derive(Debug, Clone, Copy)]
struct ParsedField<T> {
    value: T,
    line: usize,
}

#[derive(Debug, Default)]
struct GlobalFieldsBuilder {
    camera_look_at: Option<ParsedField<Point3>>,
    camera_look_from: Option<ParsedField<Point3>>,
    camera_look_up: Option<ParsedField<Direction3>>,
    field_of_view: Option<ParsedField<f64>>,
    direction_to_light: Option<ParsedField<Direction3>>,
    light_color: Option<ParsedField<ColorRgb>>,
    ambient_light: Option<ParsedField<ColorRgb>>,
    background_color: Option<ParsedField<ColorRgb>>,
}

impl GlobalFieldsBuilder {
    fn set_field(
        &mut self,
        directive: GlobalDirective,
        value_tokens: &[&str],
        line: usize,
    ) -> Result<(), SceneParseError> {
        match directive {
            GlobalDirective::CameraLookAt => {
                let value = parse_vector3(
                    GlobalDirective::CameraLookAt.field_name(),
                    value_tokens,
                    line,
                )?;
                set_once(
                    &mut self.camera_look_at,
                    value,
                    line,
                    GlobalDirective::CameraLookAt.field_name(),
                )
            }
            GlobalDirective::CameraLookFrom => {
                let value = parse_vector3(
                    GlobalDirective::CameraLookFrom.field_name(),
                    value_tokens,
                    line,
                )?;
                set_once(
                    &mut self.camera_look_from,
                    value,
                    line,
                    GlobalDirective::CameraLookFrom.field_name(),
                )
            }
            GlobalDirective::CameraLookUp => {
                let value = parse_vector3(
                    GlobalDirective::CameraLookUp.field_name(),
                    value_tokens,
                    line,
                )?;
                set_once(
                    &mut self.camera_look_up,
                    value,
                    line,
                    GlobalDirective::CameraLookUp.field_name(),
                )
            }
            GlobalDirective::FieldOfView => {
                let value = parse_scalar(
                    GlobalDirective::FieldOfView.field_name(),
                    value_tokens,
                    line,
                )?;
                set_once(
                    &mut self.field_of_view,
                    value,
                    line,
                    GlobalDirective::FieldOfView.field_name(),
                )
            }
            GlobalDirective::DirectionToLight => {
                let value = parse_vector3(
                    GlobalDirective::DirectionToLight.field_name(),
                    value_tokens,
                    line,
                )?;
                set_once(
                    &mut self.direction_to_light,
                    value,
                    line,
                    GlobalDirective::DirectionToLight.field_name(),
                )
            }
            GlobalDirective::LightColor => {
                let value =
                    parse_vector3(GlobalDirective::LightColor.field_name(), value_tokens, line)?;
                set_once(
                    &mut self.light_color,
                    value,
                    line,
                    GlobalDirective::LightColor.field_name(),
                )
            }
            GlobalDirective::AmbientLight => {
                let value = parse_vector3(
                    GlobalDirective::AmbientLight.field_name(),
                    value_tokens,
                    line,
                )?;
                set_once(
                    &mut self.ambient_light,
                    value,
                    line,
                    GlobalDirective::AmbientLight.field_name(),
                )
            }
            GlobalDirective::BackgroundColor => {
                let value = parse_vector3(
                    GlobalDirective::BackgroundColor.field_name(),
                    value_tokens,
                    line,
                )?;
                set_once(
                    &mut self.background_color,
                    value,
                    line,
                    GlobalDirective::BackgroundColor.field_name(),
                )
            }
        }
    }

    fn build_camera(&self, eof_line: usize) -> Result<Camera, SceneParseError> {
        let look_at = require_field(
            self.camera_look_at.as_ref(),
            GlobalDirective::CameraLookAt.field_name(),
            eof_line,
        )?;
        let look_from = require_field(
            self.camera_look_from.as_ref(),
            GlobalDirective::CameraLookFrom.field_name(),
            eof_line,
        )?;
        let look_up = require_field(
            self.camera_look_up.as_ref(),
            GlobalDirective::CameraLookUp.field_name(),
            eof_line,
        )?;
        let field_of_view = require_field(
            self.field_of_view.as_ref(),
            GlobalDirective::FieldOfView.field_name(),
            eof_line,
        )?;

        Camera::try_new(look_from, look_at, look_up, field_of_view)
            .map_err(|source| self.map_validation_error(source))
    }

    fn build_lighting(&self, eof_line: usize) -> Result<Lighting, SceneParseError> {
        let direction_to_light = require_field(
            self.direction_to_light.as_ref(),
            GlobalDirective::DirectionToLight.field_name(),
            eof_line,
        )?;
        let light_color = require_field(
            self.light_color.as_ref(),
            GlobalDirective::LightColor.field_name(),
            eof_line,
        )?;
        let ambient_light = require_field(
            self.ambient_light.as_ref(),
            GlobalDirective::AmbientLight.field_name(),
            eof_line,
        )?;
        let background_color = require_field(
            self.background_color.as_ref(),
            GlobalDirective::BackgroundColor.field_name(),
            eof_line,
        )?;

        Lighting::try_new(
            direction_to_light,
            light_color,
            ambient_light,
            background_color,
        )
        .map_err(|source| self.map_validation_error(source))
    }

    fn line_for_field(&self, field_name: &'static str) -> Option<usize> {
        match field_name {
            "CameraLookAt" => self.camera_look_at.as_ref().map(|field| field.line),
            "CameraLookFrom" => self.camera_look_from.as_ref().map(|field| field.line),
            "CameraLookUp" => self.camera_look_up.as_ref().map(|field| field.line),
            "FieldOfView" => self.field_of_view.as_ref().map(|field| field.line),
            "DirectionToLight" => self.direction_to_light.as_ref().map(|field| field.line),
            "LightColor" => self.light_color.as_ref().map(|field| field.line),
            "AmbientLight" => self.ambient_light.as_ref().map(|field| field.line),
            "BackgroundColor" => self.background_color.as_ref().map(|field| field.line),
            _ => None,
        }
    }

    fn map_validation_error(&self, source: SceneValidationError) -> SceneParseError {
        match source {
            SceneValidationError::CoincidentPoints {
                left_field,
                right_field,
            } => SceneParseError::ValidationAcrossLines {
                left_line: self.line_for_field(left_field).unwrap_or(1),
                right_line: self.line_for_field(right_field).unwrap_or(1),
                source: SceneValidationError::CoincidentPoints {
                    left_field,
                    right_field,
                },
            },
            SceneValidationError::NonFiniteScalar { field, value } => {
                SceneParseError::ValidationAtLine {
                    line: self.line_for_field(field).unwrap_or(1),
                    source: SceneValidationError::NonFiniteScalar { field, value },
                }
            }
            SceneValidationError::ScalarOutOfRange {
                field,
                min,
                max,
                value,
            } => SceneParseError::ValidationAtLine {
                line: self.line_for_field(field).unwrap_or(1),
                source: SceneValidationError::ScalarOutOfRange {
                    field,
                    min,
                    max,
                    value,
                },
            },
            SceneValidationError::NonPositiveScalar { field, value } => {
                SceneParseError::ValidationAtLine {
                    line: self.line_for_field(field).unwrap_or(1),
                    source: SceneValidationError::NonPositiveScalar { field, value },
                }
            }
            SceneValidationError::NonFiniteVector { field, x, y, z } => {
                SceneParseError::ValidationAtLine {
                    line: self.line_for_field(field).unwrap_or(1),
                    source: SceneValidationError::NonFiniteVector { field, x, y, z },
                }
            }
            SceneValidationError::ZeroLengthVector { field } => SceneParseError::ValidationAtLine {
                line: self.line_for_field(field).unwrap_or(1),
                source: SceneValidationError::ZeroLengthVector { field },
            },
        }
    }
}

#[derive(Debug)]
struct SphereFieldsBuilder {
    start_line: usize,
    center: Option<ParsedField<Point3>>,
    radius: Option<ParsedField<f64>>,
    kd: Option<ParsedField<f64>>,
    ks: Option<ParsedField<f64>>,
    ka: Option<ParsedField<f64>>,
    od: Option<ParsedField<ColorRgb>>,
    os: Option<ParsedField<ColorRgb>>,
    kgls: Option<ParsedField<f64>>,
}

impl SphereFieldsBuilder {
    fn new(start_line: usize) -> Self {
        Self {
            start_line,
            center: None,
            radius: None,
            kd: None,
            ks: None,
            ka: None,
            od: None,
            os: None,
            kgls: None,
        }
    }

    fn set_field(
        &mut self,
        directive: SphereDirective,
        value_tokens: &[&str],
        line: usize,
    ) -> Result<(), SceneParseError> {
        match directive {
            SphereDirective::Center => {
                let value =
                    parse_vector3(SphereDirective::Center.field_name(), value_tokens, line)?;
                set_once(
                    &mut self.center,
                    value,
                    line,
                    SphereDirective::Center.field_name(),
                )
            }
            SphereDirective::Radius => {
                let value = parse_scalar(SphereDirective::Radius.field_name(), value_tokens, line)?;
                set_once(
                    &mut self.radius,
                    value,
                    line,
                    SphereDirective::Radius.field_name(),
                )
            }
            SphereDirective::Kd => {
                let value = parse_scalar(SphereDirective::Kd.field_name(), value_tokens, line)?;
                set_once(&mut self.kd, value, line, SphereDirective::Kd.field_name())
            }
            SphereDirective::Ks => {
                let value = parse_scalar(SphereDirective::Ks.field_name(), value_tokens, line)?;
                set_once(&mut self.ks, value, line, SphereDirective::Ks.field_name())
            }
            SphereDirective::Ka => {
                let value = parse_scalar(SphereDirective::Ka.field_name(), value_tokens, line)?;
                set_once(&mut self.ka, value, line, SphereDirective::Ka.field_name())
            }
            SphereDirective::Od => {
                let value = parse_vector3(SphereDirective::Od.field_name(), value_tokens, line)?;
                set_once(&mut self.od, value, line, SphereDirective::Od.field_name())
            }
            SphereDirective::Os => {
                let value = parse_vector3(SphereDirective::Os.field_name(), value_tokens, line)?;
                set_once(&mut self.os, value, line, SphereDirective::Os.field_name())
            }
            SphereDirective::Kgls => {
                let value = parse_scalar(SphereDirective::Kgls.field_name(), value_tokens, line)?;
                set_once(
                    &mut self.kgls,
                    value,
                    line,
                    SphereDirective::Kgls.field_name(),
                )
            }
        }
    }

    fn build_sphere(self) -> Result<Sphere, SceneParseError> {
        let center = require_field(
            self.center.as_ref(),
            SphereDirective::Center.field_name(),
            self.start_line,
        )?;
        let radius = require_field(
            self.radius.as_ref(),
            SphereDirective::Radius.field_name(),
            self.start_line,
        )?;
        let kd = require_field(
            self.kd.as_ref(),
            SphereDirective::Kd.field_name(),
            self.start_line,
        )?;
        let ks = require_field(
            self.ks.as_ref(),
            SphereDirective::Ks.field_name(),
            self.start_line,
        )?;
        let ka = require_field(
            self.ka.as_ref(),
            SphereDirective::Ka.field_name(),
            self.start_line,
        )?;
        let od = require_field(
            self.od.as_ref(),
            SphereDirective::Od.field_name(),
            self.start_line,
        )?;
        let os = require_field(
            self.os.as_ref(),
            SphereDirective::Os.field_name(),
            self.start_line,
        )?;
        let kgls = require_field(
            self.kgls.as_ref(),
            SphereDirective::Kgls.field_name(),
            self.start_line,
        )?;

        let material = Material::try_new(kd, ks, ka, od, os, kgls)
            .map_err(|source| self.map_validation_error(source))?;

        Sphere::try_new(center, radius, material)
            .map_err(|source| self.map_validation_error(source))
    }

    fn line_for_field(&self, field_name: &'static str) -> Option<usize> {
        match field_name {
            "Center" => self.center.as_ref().map(|field| field.line),
            "Radius" => self.radius.as_ref().map(|field| field.line),
            "Kd" => self.kd.as_ref().map(|field| field.line),
            "Ks" => self.ks.as_ref().map(|field| field.line),
            "Ka" => self.ka.as_ref().map(|field| field.line),
            "Od" => self.od.as_ref().map(|field| field.line),
            "Os" => self.os.as_ref().map(|field| field.line),
            "Kgls" => self.kgls.as_ref().map(|field| field.line),
            _ => None,
        }
    }

    fn map_validation_error(&self, source: SceneValidationError) -> SceneParseError {
        match source {
            SceneValidationError::CoincidentPoints {
                left_field,
                right_field,
            } => SceneParseError::ValidationAcrossLines {
                left_line: self.line_for_field(left_field).unwrap_or(self.start_line),
                right_line: self.line_for_field(right_field).unwrap_or(self.start_line),
                source: SceneValidationError::CoincidentPoints {
                    left_field,
                    right_field,
                },
            },
            SceneValidationError::NonFiniteScalar { field, value } => {
                SceneParseError::ValidationAtLine {
                    line: self.line_for_field(field).unwrap_or(self.start_line),
                    source: SceneValidationError::NonFiniteScalar { field, value },
                }
            }
            SceneValidationError::ScalarOutOfRange {
                field,
                min,
                max,
                value,
            } => SceneParseError::ValidationAtLine {
                line: self.line_for_field(field).unwrap_or(self.start_line),
                source: SceneValidationError::ScalarOutOfRange {
                    field,
                    min,
                    max,
                    value,
                },
            },
            SceneValidationError::NonPositiveScalar { field, value } => {
                SceneParseError::ValidationAtLine {
                    line: self.line_for_field(field).unwrap_or(self.start_line),
                    source: SceneValidationError::NonPositiveScalar { field, value },
                }
            }
            SceneValidationError::NonFiniteVector { field, x, y, z } => {
                SceneParseError::ValidationAtLine {
                    line: self.line_for_field(field).unwrap_or(self.start_line),
                    source: SceneValidationError::NonFiniteVector { field, x, y, z },
                }
            }
            SceneValidationError::ZeroLengthVector { field } => SceneParseError::ValidationAtLine {
                line: self.line_for_field(field).unwrap_or(self.start_line),
                source: SceneValidationError::ZeroLengthVector { field },
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlobalDirective {
    CameraLookAt,
    CameraLookFrom,
    CameraLookUp,
    FieldOfView,
    DirectionToLight,
    LightColor,
    AmbientLight,
    BackgroundColor,
}

impl GlobalDirective {
    fn from_token(token: &str) -> Option<Self> {
        match token {
            "CameraLookAt" => Some(Self::CameraLookAt),
            "CameraLookFrom" => Some(Self::CameraLookFrom),
            "CameraLookUp" => Some(Self::CameraLookUp),
            "FieldOfView" => Some(Self::FieldOfView),
            "DirectionToLight" => Some(Self::DirectionToLight),
            "LightColor" => Some(Self::LightColor),
            "AmbientLight" => Some(Self::AmbientLight),
            "BackgroundColor" => Some(Self::BackgroundColor),
            _ => None,
        }
    }

    fn field_name(self) -> &'static str {
        match self {
            Self::CameraLookAt => "CameraLookAt",
            Self::CameraLookFrom => "CameraLookFrom",
            Self::CameraLookUp => "CameraLookUp",
            Self::FieldOfView => "FieldOfView",
            Self::DirectionToLight => "DirectionToLight",
            Self::LightColor => "LightColor",
            Self::AmbientLight => "AmbientLight",
            Self::BackgroundColor => "BackgroundColor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SphereDirective {
    Center,
    Radius,
    Kd,
    Ks,
    Ka,
    Od,
    Os,
    Kgls,
}

impl SphereDirective {
    fn from_token(token: &str) -> Option<Self> {
        match token {
            "Center" => Some(Self::Center),
            "Radius" => Some(Self::Radius),
            "Kd" => Some(Self::Kd),
            "Ks" => Some(Self::Ks),
            "Ka" => Some(Self::Ka),
            "Od" => Some(Self::Od),
            "Os" => Some(Self::Os),
            "Kgls" => Some(Self::Kgls),
            _ => None,
        }
    }

    fn field_name(self) -> &'static str {
        match self {
            Self::Center => "Center",
            Self::Radius => "Radius",
            Self::Kd => "Kd",
            Self::Ks => "Ks",
            Self::Ka => "Ka",
            Self::Od => "Od",
            Self::Os => "Os",
            Self::Kgls => "Kgls",
        }
    }
}

#[derive(Debug, Default)]
struct SceneTextParser {
    globals: GlobalFieldsBuilder,
    current_sphere: Option<SphereFieldsBuilder>,
    spheres: Vec<Sphere>,
    eof_line: usize,
}

impl SceneTextParser {
    fn new() -> Self {
        Self::default()
    }

    fn parse_line(&mut self, line_number: usize, raw_line: &str) -> Result<(), SceneParseError> {
        self.eof_line = line_number;

        let normalized_line = normalize_input_line(raw_line);
        if normalized_line.trim().is_empty() {
            return Ok(());
        }

        let tokens: Vec<&str> = normalized_line.split_whitespace().collect();
        let directive_token = tokens[0];
        let value_tokens = &tokens[1..];

        if directive_token == "Sphere" {
            if !value_tokens.is_empty() {
                return Err(SceneParseError::SphereDirectiveHasValues { line: line_number });
            }

            self.finish_current_sphere_if_any()?;
            self.current_sphere = Some(SphereFieldsBuilder::new(line_number));
            return Ok(());
        }

        if let Some(global_directive) = GlobalDirective::from_token(directive_token) {
            if self.current_sphere.is_some() {
                return Err(SceneParseError::GlobalFieldInsideSphere {
                    line: line_number,
                    field: global_directive.field_name(),
                });
            }

            return self
                .globals
                .set_field(global_directive, value_tokens, line_number);
        }

        if let Some(sphere_directive) = SphereDirective::from_token(directive_token) {
            let current_sphere =
                self.current_sphere
                    .as_mut()
                    .ok_or(SceneParseError::SphereFieldBeforeSphere {
                        line: line_number,
                        field: sphere_directive.field_name(),
                    })?;

            return current_sphere.set_field(sphere_directive, value_tokens, line_number);
        }

        Err(SceneParseError::UnknownDirective {
            line: line_number,
            directive: directive_token.to_string(),
        })
    }

    fn finish(mut self) -> Result<Scene, SceneParseError> {
        self.finish_current_sphere_if_any()?;

        let eof_line = self.eof_line.max(1);
        let camera = self.globals.build_camera(eof_line)?;
        let lighting = self.globals.build_lighting(eof_line)?;
        Ok(Scene::new(camera, lighting, self.spheres))
    }

    fn finish_current_sphere_if_any(&mut self) -> Result<(), SceneParseError> {
        if let Some(current_sphere) = self.current_sphere.take() {
            let sphere = current_sphere.build_sphere()?;
            self.spheres.push(sphere);
        }

        Ok(())
    }
}

fn normalize_input_line(raw_line: &str) -> String {
    raw_line
        .split_once('#')
        .map_or(raw_line, |(before_comment, _)| before_comment)
        .replace(',', " ")
}

fn parse_scalar(
    field: &'static str,
    value_tokens: &[&str],
    line: usize,
) -> Result<f64, SceneParseError> {
    if value_tokens.len() != 1 {
        return Err(SceneParseError::WrongValueCount {
            line,
            field,
            expected: 1,
            found: value_tokens.len(),
        });
    }

    value_tokens[0]
        .parse::<f64>()
        .map_err(|_| SceneParseError::InvalidNumber {
            line,
            field,
            token: value_tokens[0].to_string(),
        })
}

fn parse_vector3(
    field: &'static str,
    value_tokens: &[&str],
    line: usize,
) -> Result<Point3, SceneParseError> {
    if value_tokens.len() != 3 {
        return Err(SceneParseError::WrongValueCount {
            line,
            field,
            expected: 3,
            found: value_tokens.len(),
        });
    }

    let x = parse_number_token(field, value_tokens[0], line)?;
    let y = parse_number_token(field, value_tokens[1], line)?;
    let z = parse_number_token(field, value_tokens[2], line)?;
    Ok(Point3::new(x, y, z))
}

fn parse_number_token(
    field: &'static str,
    token: &str,
    line: usize,
) -> Result<f64, SceneParseError> {
    token
        .parse::<f64>()
        .map_err(|_| SceneParseError::InvalidNumber {
            line,
            field,
            token: token.to_string(),
        })
}

fn set_once<T: Copy>(
    slot: &mut Option<ParsedField<T>>,
    value: T,
    line: usize,
    field: &'static str,
) -> Result<(), SceneParseError> {
    if let Some(existing) = slot.as_ref() {
        return Err(SceneParseError::DuplicateField {
            line,
            field,
            first_defined_line: existing.line,
        });
    }

    *slot = Some(ParsedField { value, line });
    Ok(())
}

fn require_field<T: Copy>(
    value: Option<&ParsedField<T>>,
    field: &'static str,
    eof_line: usize,
) -> Result<T, SceneParseError> {
    value
        .map(|parsed| parsed.value)
        .ok_or(SceneParseError::MissingRequiredField {
            line: eof_line,
            field,
        })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn parses_fixture_scene_1() {
        let path = Path::new("../raytracer-scene_1.txt");
        let scene = parse_scene_file(path).expect("fixture scene 1 should parse");

        assert_eq!(scene.spheres().len(), 1);
        assert_eq!(scene.camera().field_of_view_degrees(), 90.0);
        assert_eq!(
            scene.lighting().background_color(),
            ColorRgb::new(0.2, 0.2, 0.2)
        );
    }

    #[test]
    fn parses_fixture_scene_2_with_commas() {
        let path = Path::new("../raytracer-scene_2.txt");
        let scene = parse_scene_file(path).expect("fixture scene 2 should parse");

        assert_eq!(scene.spheres().len(), 4);
        assert_eq!(
            scene.spheres()[0].material().diffuse_color(),
            ColorRgb::new(1.0, 1.0, 1.0)
        );
    }

    #[test]
    fn unknown_directive_reports_line_number() {
        let source = "CameraLookAt 0 0 0\nNotARealDirective 1 2 3\n";
        let error = parse_scene_text(source).expect_err("unknown directive should fail");

        assert!(matches!(
            error,
            SceneParseError::UnknownDirective { line: 2, .. }
        ));
    }

    #[test]
    fn duplicate_field_reports_both_lines() {
        let source = "CameraLookAt 0 0 0\nCameraLookAt 1 2 3\n";
        let error = parse_scene_text(source).expect_err("duplicate field should fail");

        assert!(matches!(
            error,
            SceneParseError::DuplicateField {
                line: 2,
                first_defined_line: 1,
                ..
            }
        ));
    }

    #[test]
    fn sphere_field_before_sphere_reports_line() {
        let source = "Center 0 0 0\n";
        let error = parse_scene_text(source).expect_err("sphere field before sphere should fail");

        assert!(matches!(
            error,
            SceneParseError::SphereFieldBeforeSphere { line: 1, .. }
        ));
    }

    #[test]
    fn global_field_inside_sphere_reports_line() {
        let source = "Sphere\nCenter 0 0 0\nCameraLookAt 0 0 0\n";
        let error = parse_scene_text(source).expect_err("global field inside sphere should fail");

        assert!(matches!(
            error,
            SceneParseError::GlobalFieldInsideSphere { line: 3, .. }
        ));
    }
}
