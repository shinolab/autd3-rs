use std::{fmt::Debug, hash::Hash};

use super::OperationGenerator;
use crate::{datagram::GroupOpGenerator, firmware::driver::Version};

use autd3_core::geometry::Device;

impl<'dev, K, F, G> OperationGenerator<'dev> for GroupOpGenerator<K, F, G>
where
    K: Hash + Eq + Debug,
    F: Fn(&Device) -> Option<K>,
    G: OperationGenerator<'dev>,
{
    type O1 = <G as OperationGenerator<'dev>>::O1;
    type O2 = <G as OperationGenerator<'dev>>::O2;

    fn generate(&mut self, dev: &'dev Device, version: Version) -> Option<(Self::O1, Self::O2)> {
        let key = (self.key_map)(dev)?;
        self.generators
            .get_mut(&key)
            .and_then(|g| g.generate(dev, version))
    }
}
