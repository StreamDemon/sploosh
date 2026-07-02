/// Trait and impl shapes: generics, supertraits, where clauses, trait impls,
/// and `dyn` trait references with type arguments (§16 trait_def, impl_block,
/// trait_ref).
trait Printable {
    fn print(&self);
}

trait Convert<T> {
    fn convert(&self) -> T;
}

trait Loggable: Printable {
    fn log(&self);
}

trait Bounded: Convert<i64> + Printable where Self: Printable {
    fn cap(&self) -> i64;
}

struct User {
    id: u64,
}

struct Wrapper<T> {
    value: T,
}

impl User {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Printable for User {
    fn print(&self) {}
}

impl<T> Convert<T> for Wrapper<T> where T: Clone {
    fn convert(&self) -> T {
        self.value
    }
}

impl Loggable for User where User: Printable {
    fn log(&self) {}
}

struct Holder {
    it: Box<dyn Iter<Item = i64>>,
}

fn takes(c: &dyn Convert<i64>) {}
