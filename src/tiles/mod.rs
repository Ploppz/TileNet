use std::fmt;
use std::cmp::min;
pub use self::tilenet::{TileNet, TileNetProxy};

mod tilenet;

/// Tile iterator returning tiles from the `tile_net::TileNet`.
///
/// ```
/// use tile_net::{Line, Vector, TileNet};
///
/// let map: TileNet<usize> = TileNet::sample();
/// let iter = (0..10).map(|x| (x, 3));
/// let set = map.collide_set(iter);
/// set.map(|x| println!("{:?}", x)).count();
///
/// let map = TileNet::sample();
/// let line = Line::from_origin(Vector(10.0, 5.0));
/// let cover = line.supercover();
/// let set = map.collide_set(cover);
/// for tile in set {
/// 	println!("{:?}", tile);
/// }
/// ```
///
/// The ideal version of this library is quite simple:
///
/// ```ignore
/// // Pseudocode
/// use tile_net::{TileNet, Line};
/// let map: TileNet<MyTile> = TileNet::new((1000, 1000));
/// initialize_map(&mut map);
/// 'main: loop {
/// 	handle_events(&mut world);
/// 	// Physics for collidable units
/// 		run!(
/// 		{
/// 			for coll in world.collidables_mut() {
/// 				match map.collides(coll) {
/// 					TileNet::NoCollision => coll.allow_move(),
/// 					TileNet::Collision(collset) => coll.deny_move(&collset),
/// 				}
/// 			}
///   } until TileBased::is_resolved(&coll));
/// }
/// ```
#[derive(Clone)]
pub struct TileSet<'a, T, I>
	where T: 'a
{
	tilenet: &'a TileNet<T>,
	points: I,
	last_coord: (i32, i32),
}

impl<'a, T, I> TileSet<'a, T, I>
	where T: 'a,
	      I: Iterator<Item = (i32, i32)>
{
	/// Get the coordinate of the last tile
	///
	/// When iterating over the tileset, it may be useful to get the last
	/// coordinate. This function provides you this coordinate.
	pub fn get_last_coord(&self) -> (i32, i32) {
		self.last_coord
	}
}

impl<'a, T, I> Iterator for TileSet<'a, T, I>
	where T: 'a,
	      I: Iterator<Item = (i32, i32)>
{
	type Item = &'a T;
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some(point) = self.points.next() {
				self.last_coord = point;
				if point.0 >= 0 && point.1 >= 0 && point.0 < self.tilenet.get_size().0 as i32 &&
				   point.1 < self.tilenet.get_size().1 as i32 {
					return self.tilenet.get((point.0 as usize, point.1 as usize));
				} else {
					continue;
				}
			} else {
				return None;
			}
		}
	}
}

impl<'a, T, I> fmt::Debug for TileSet<'a, T, I>
	where T: 'a + Clone + fmt::Debug,
	      I: Clone + Iterator<Item = (i32, i32)>
{
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		let viewer = self.clone();
		for tile in viewer {
			try!(write!(formatter, "{:?} ", tile));
		}
		Ok(())
	}
}

/// Tile iterator for a rectangular view of the `tile_net::TileNet`.
///
/// Used to cull the amount of tiles to draw. You provide it with a desired
/// rectangle, and the tileview will be your iterator iterating over only
/// the desired tiles.
#[derive(Clone)]
pub struct TileView<'a, T>
	where T: 'a
{
	tilenet: &'a TileNet<T>,
	rectangle: (usize, usize, usize, usize),
	current: (usize, usize),
}

impl<'a, T> TileView<'a, T>
    where T: 'a
{
	fn new(tilenet: &'a TileNet<T>,
	       mut rectangle: (usize, usize, usize, usize))
	       -> TileView<'a, T> {
		rectangle.1 = min(rectangle.1, tilenet.get_size().0);
		rectangle.3 = min(rectangle.3, tilenet.get_size().1);
		TileView {
			tilenet: tilenet,
			rectangle: rectangle,
			current: (rectangle.0, rectangle.2),
		}
	}
}

impl<'a, T> Iterator for TileView<'a, T>
    where T: 'a
{
	type Item = (&'a T, usize, usize);
	fn next(&mut self) -> Option<Self::Item> {
		if self.current.1 >= self.rectangle.3 {
			return None;
		}
		let tile = self.tilenet.get(self.current).map(|x| (x, self.current.0, self.current.1));

		self.current.0 += 1;
		if self.current.0 >= self.rectangle.1 {
			self.current.1 += 1;
			self.current.0 = self.rectangle.0;
		}
		tile
	}
}

impl<'a, T> fmt::Debug for TileView<'a, T>
    where T: 'a + Clone + fmt::Debug
{
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		let biggest = self.clone().map(|x| format!("{:?}", x).len()).max();
		let viewer = self.clone();
		let width = viewer.rectangle.1 - viewer.rectangle.0;
		for (index, tile) in viewer.enumerate() {
			if index % width == 0 && index != 0 {
				try!(formatter.write_str("\n"));
			}
			let mut current = format!("{:?}", tile);
			let length = current.len();
			if let Some(biggest) = biggest {
				(0..biggest - length).map(|_| current.push(' ')).count();
			}
			try!(write!(formatter, "{} ", current));
		}
		Ok(())
	}
}
