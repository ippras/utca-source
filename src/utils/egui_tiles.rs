use egui_tiles::{Container, Tile, TileId, Tiles, Tree};

/// [`Container`] extension methods
pub trait ContainerExt {
    fn find_child_pane<'a, T>(&'a self, tiles: &'a Tiles<T>) -> Option<&'a T>;

    // fn active_panes<'a, T>(&'a self, tiles: &'a Tiles<T>, f: impl Fn(&T));
}

impl ContainerExt for Container {
    fn find_child_pane<'a, T>(&'a self, tiles: &'a Tiles<T>) -> Option<&'a T> {
        self.children().find_map(|child| match tiles.get(*child)? {
            Tile::Container(container) => container.find_child_pane(tiles),
            Tile::Pane(pane) => Some(pane),
        })
    }

    // fn active_panes<'a, T>(&'a self, tiles: &'a Tiles<T>, f: impl Fn(&T)) {
    //     for child in self.active_children() {
    //         match tiles.get(*child).unwrap() {
    //             Tile::Container(container) => container.active_panes(tiles, &f),
    //             Tile::Pane(pane) => f(pane),
    //         }
    //     }
    // }
}

/// Extension methods for [`Tiles`]
pub trait TilesExt<T> {
    fn find_pane_by(&mut self, f: impl Fn(&T) -> bool) -> Option<TileId>;
}

impl<T> TilesExt<T> for Tiles<T> {
    fn find_pane_by(&mut self, f: impl Fn(&T) -> bool) -> Option<TileId> {
        self.iter()
            .find(|(_, tile)| {
                if let Tile::Pane(pane) = *tile {
                    f(pane)
                } else {
                    false
                }
            })
            .map(|(tile_id, _)| *tile_id)
    }
}

/// [`Tree`] extension methods
pub trait TreeExt<T> {
    fn insert_pane(&mut self, pane: T);
}

impl<T> TreeExt<T> for Tree<T> {
    fn insert_pane(&mut self, pane: T) {
        let child = self.tiles.insert_pane(pane);
        if let Some(root) = self.root {
            if let Some(tile) = self.tiles.get_mut(root) {
                if let Tile::Container(container) = tile {
                    container.add_child(child);
                } else {
                    self.root = Some(self.tiles.insert_vertical_tile(vec![root, child]));
                }
            }
        } else {
            self.root = Some(child)
        }
    }
}
