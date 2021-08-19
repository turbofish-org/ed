use ed::Decode;

#[derive(Decode)]
struct Foo {
  x: u32,
  y: (u32, u32),
}

#[derive(Decode)]
struct Foo2(u32, (u32, u32));

#[derive(Decode)]
struct Foo3;

#[derive(Decode)]
enum Bar {
  A {
    x: u32,
    y: (u32, u32),
  },
  B(u32, (u32, u32)),
  C,
}
