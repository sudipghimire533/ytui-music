use ratatui::layout::{Constraint, Direction, Layout, Rect};

/*

|‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾|
|                top bar                                                 |
|________________________________________________________________________|
|           |                                                            |
|           |                                                            |
|           |                                                            |
|           |                                                            |
|           |                                                            |
|           |                                                            |
|    side   |                                                            |
|    bar    |                   main area                                |
|           |                                                            |
|           |                                                            |
|           |                                                            |
|           |                                                            |
|           |                                                            |
|           |                                                            |
|           |                                                            |
| ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾|
|                            progress bar                                |
|________________________________________________________________________|

======
    - side bar can be in left or right

====== top bar:
|‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾|
|       searchbar                                       |     status     |
|________________________________________________________________________|


==== sidebar
|‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾|
|                   |
|                   |
|     shortcuts     |
|                   |
|                   |
|                   |
|                   |
|-------------------|
|                   |
|                   |
|                   |
|   next in         |
|   queue           |
|                   |
|                   |
|___________________|
*/

pub struct Dimension {
    pub searchbar: Rect,
    pub statusbar: (Rect, [Rect; 4]),
}

pub struct DimensionArgs;
impl DimensionArgs {
    pub fn calculate_dimension(&self, frame_area: Rect) -> Dimension {
        let [top_area, middle_area, bottom_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .split(frame_area)[..]
            .try_into()
            .expect("Always splitted to 3");

        let [search_area, status_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(35)])
            .split(top_area)[..]
            .try_into()
            .expect("always split to 2");

        let [sidebar, main_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(middle_area)[..]
            .try_into()
            .expect("always split to 2");

        let status_components = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1); 4])
            .split(status_area)
            .iter()
            .copied()
            .map(|mut rect| {
                rect.x += 1;
                rect.y += 1;
                rect
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Dimension {
            searchbar: search_area,
            statusbar: (status_area, status_components),
        }
    }
}
