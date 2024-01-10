// pub mod __implmentation_basichybrid {
//     use ::limousine_engine::private::*;
//     type A0 = PGMAddress;
//     type A1 = PGMAddress;
//     type A2 = ();
//     type C0<K, V> = MemoryPGMLayer<K, V, 64usize, A1>;
//     type C1<K, V> = MemoryPGMLayer<K, V, 16usize, A2>;
//     type C2<K, V> = BTreeTopComponent<K, V, A1>;
//     pub struct BasicHybrid<K: Key, V: Value> {
//         pub c0: C0<K, V>,
//         pub c1: C1<K, V>,
//         pub c2: C2<K, V>,
//     }
//     impl<K: Key, V: Value> BasicHybrid<K, V> {
//         pub fn search(&self, key: &K) -> Option<&V> {
//             let s2 = self.c2.search(&self.c1, &key);
//             let s1 = self.c1.search(&self.c0, s2, &key);
//             let s0 = self.c0.search(s1, &key);
//             s0
//         }
//         pub fn insert(&mut self, key: K, value: V) -> Option<V> {
//             let s2 = self.c2.search(&self.c1, &key);
//             let s1 = self.c1.search(&self.c0, s2, &key);
//             let s0 = self.c0.search(s1, &key);
//             let result = s0.copied();
//             let i0;
//             if let Some(x) = self.c0.insert(s1, key, value) {
//                 i0 = x;
//             } else {
//                 return result;
//             }
//             let i1;
//             if let Some(x) = self.c1.insert(&mut self.c0, i0) {
//                 i1 = x;
//             } else {
//                 return result;
//             }
//             let i2 = self.c2.insert(&mut self.c1, i1);
//             result
//         }
//         pub fn empty() -> Self {
//             let mut c0 = C0::empty();
//             let mut c1 = C1::build(&mut c0);
//             let mut c2 = C2::build(&mut c1);
//             Self { c0, c1, c2 }
//         }
//         pub fn build(iter: impl Iterator<Item = Entry<K, V>>) -> Self {
//             let mut c0 = C0::build(iter);
//             let mut c1 = C1::build(&mut c0);
//             let mut c2 = C2::build(&mut c1);
//             Self { c0, c1, c2 }
//         }
//     }
// }
// use __implmentation_basichybrid::BasicHybrid;
