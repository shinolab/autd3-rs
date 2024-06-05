mod csv;
mod error;
mod rawpcm;
mod wav;

pub use csv::Csv;
pub use rawpcm::RawPCM;
pub use wav::Wav;

#[cfg(test)]
mod tests {
    use autd3_driver::{
        autd3_device::AUTD3,
        defined::FREQ_40K,
        derive::Geometry,
        geometry::{IntoDevice, Vector3},
    };

    pub fn create_geometry(n: usize) -> Geometry {
        Geometry::new(
            (0..n)
                .map(|i| AUTD3::new(Vector3::zeros()).into_device(i))
                .collect(),
            FREQ_40K,
        )
    }
}
