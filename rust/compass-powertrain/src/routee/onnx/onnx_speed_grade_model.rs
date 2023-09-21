use crate::routee::prediction_model::SpeedGradePredictionModel;
use compass_core::{
    model::traversal::traversal_model_error::TraversalModelError,
    util::unit::{as_f64::AsF64, EnergyRate, EnergyRateUnit, Speed, SpeedUnit},
};
use ndarray::CowArray;
use ort::{
    tensor::OrtOwnedTensor, Environment, GraphOptimizationLevel, Session, SessionBuilder, Value,
};

pub struct OnnxSpeedGradeModel {
    session: Session,
    speed_unit: SpeedUnit,
    energy_rate_unit: EnergyRateUnit,
}

impl SpeedGradePredictionModel for OnnxSpeedGradeModel {
    fn predict(
        &self,
        speed: Speed,
        speed_unit: SpeedUnit,
        grade: f64,
    ) -> Result<(EnergyRate, EnergyRateUnit), TraversalModelError> {
        let speed_value: f32 = speed_unit.convert(speed, self.speed_unit.clone()).as_f64() as f32;
        let grade_value: f32 = grade as f32;
        let array = ndarray::Array1::from(vec![speed_value, grade_value])
            .into_shape((1, 2))
            .map_err(|e| {
                TraversalModelError::PredictionModel(format!(
                    "Failed to reshape input for prediction: {}",
                    e.to_string()
                ))
            })?;

        let x = CowArray::from(array).into_dyn();
        let value = Value::from_array(self.session.allocator(), &x).map_err(|e| {
            TraversalModelError::PredictionModel(format!(
                "Failed to create input value for prediction: {}",
                e.to_string()
            ))
        })?;
        let input = vec![value];

        let result: OrtOwnedTensor<f32, _> =
            self.session.run(input).unwrap()[0].try_extract().unwrap();
        let output_f64 = result.view().to_owned().into_raw_vec()[0] as f64;

        let energy_rate = EnergyRate::new(output_f64);
        Ok((energy_rate, self.energy_rate_unit.clone()))
    }
}

impl OnnxSpeedGradeModel {
    pub fn new(
        onnx_model_path: String,
        speed_unit: SpeedUnit,
        energy_rate_unit: EnergyRateUnit,
    ) -> Result<Self, TraversalModelError> {
        let env = Environment::builder()
            .build()
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
            .into_arc();

        let session = SessionBuilder::new(&env)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
            .with_intra_threads(1)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level1)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?
            .with_model_from_file(onnx_model_path)
            .map_err(|e| TraversalModelError::BuildError(e.to_string()))?;

        Ok(OnnxSpeedGradeModel {
            session,
            speed_unit,
            energy_rate_unit,
        })
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::routee::{
        onnx::onnx_speed_grade_model::OnnxSpeedGradeModel,
        prediction_model::SpeedGradePredictionModel,
    };
    use compass_core::{
        model::traversal::traversal_model_error::TraversalModelError,
        util::unit::{EnergyRate, EnergyRateUnit, Speed, SpeedUnit},
    };
    use rayon::prelude::*;

    #[test]
    // test that we can safely call this traversal model from multiple threads
    // since we have to implement unsafe code around the onnx runtime
    fn test_thread_saftey() {
        let model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("Toyota_Camry.onnx")
            .to_str()
            .unwrap()
            .into();
        let model: Box<dyn SpeedGradePredictionModel> = Box::new(
            OnnxSpeedGradeModel::new(
                model_file_path,
                SpeedUnit::MilesPerHour,
                compass_core::util::unit::EnergyRateUnit::GallonsGasolinePerMile,
            )
            .unwrap(),
        );

        let input_speed = Speed::new(50.0);
        let input_speed_unit = SpeedUnit::MilesPerHour;
        let input_grade = 0.0;
        let input_row = (
            input_speed.clone(),
            input_speed_unit.clone(),
            input_grade.clone(),
        );
        let inputs: Vec<(Speed, SpeedUnit, f64)> = (0..1000).map(|_i| input_row.clone()).collect();

        // map the model.get_energy function over the inputs using rayon
        let results = inputs
            .par_iter()
            .map(|(speed, speed_unit, grade)| model.predict(*speed, speed_unit.clone(), *grade))
            .collect::<Vec<Result<(EnergyRate, EnergyRateUnit), TraversalModelError>>>();

        // assert that all of the results are Ok
        assert!(results.iter().all(|r| r.is_ok()));

        // assert that all the results are the same
        let (expected_er, expected_eru) = model
            .predict(input_speed, input_speed_unit, input_grade)
            .unwrap();
        assert!(results.iter().all(|r| match r {
            Err(e) => panic!("{}", e),
            Ok((er, eru)) => {
                er.to_owned() == expected_er && eru.to_owned() == expected_eru
            }
        }));
    }
}
