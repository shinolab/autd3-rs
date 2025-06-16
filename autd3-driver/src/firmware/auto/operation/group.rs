use std::{fmt::Debug, hash::Hash};

use super::{super::Version, OperationGenerator};
use crate::{datagram::GroupOpGenerator, geometry::Device};

use autd3_core::datagram::Datagram;

impl<K, F, D> OperationGenerator for GroupOpGenerator<K, F, D>
where
    K: Hash + Eq + Debug,
    F: Fn(&Device) -> Option<K>,
    D: Datagram,
    D::G: OperationGenerator,
{
    type O1 = <D::G as OperationGenerator>::O1;
    type O2 = <D::G as OperationGenerator>::O2;

    fn generate(&mut self, dev: &Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        let key = (self.key_map)(dev)?;
        self.generators
            .get_mut(&key)
            .and_then(|g| g.generate(dev, version))
    }
}
