pub(crate) mod gb_sequence;
pub(crate) mod unicode;

pub trait VariableLengthEncoding: Sized {
    type Point;

    fn build(input: Self::Point) -> Option<Self>;
    fn is_complete(&self) -> bool;
    fn add_point(&mut self, point: Self::Point) -> bool;
    fn is_valid(&self) -> bool;
}
