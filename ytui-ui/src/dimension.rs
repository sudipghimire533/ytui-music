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
    pub window_border: Rect,

    pub searchbar: Rect,
    pub statusbar: (Rect, [Rect; 4]),
    pub navigation_list: Rect,
    pub queue_list: Rect,
    pub state_badge: Rect,
    pub progressbar: Rect,
    pub music_pane: Rect,
    pub playlist_pane: Rect,
    pub artist_pane: Rect,

    pub overlay: Rect,
}

pub struct DimensionArgs;
impl DimensionArgs {
    const MAX_WINDOW_HEIGHT: u16 = 70;
    const MAX_WINDOW_WIDTH: u16 = 300;

    pub fn calculate_dimension(&self, frame_area: Rect) -> Dimension {
        let [height_trimmed_area, _vertical_leftover] = Layout::default()
            .flex(ratatui::layout::Flex::Center)
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(Self::MAX_WINDOW_HEIGHT), Constraint::Max(0)])
            .split(frame_area)[..]
            .try_into()
            .expect("will always split to two");
        let [trimmed_area, _horizontal_leftover] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(ratatui::layout::Flex::Center)
            .constraints([Constraint::Max(Self::MAX_WINDOW_WIDTH), Constraint::Max(0)])
            .split(height_trimmed_area)[..]
            .try_into()
            .expect("will always split to two");

        let window_border = Rect {
            height: trimmed_area.height + 2,
            width: trimmed_area.width + 2,
            x: trimmed_area.x.saturating_sub(1),
            y: trimmed_area.y.saturating_sub(1),
        };

        let overlay = {
            let height = if trimmed_area.height < 25 {
                trimmed_area.height
            } else {
                25
            };
            let width = if trimmed_area.width < 100 {
                trimmed_area.width
            } else {
                100
            };

            Rect {
                height,
                width,
                x: (trimmed_area.width - width) / 2,
                y: (trimmed_area.height - height) / 2,
            }
        };

        let [top_area, middle_area, bottom_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .split(trimmed_area)[..]
            .try_into()
            .expect("Always splitted to 3");

        let [search_area, status_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(22)])
            .split(top_area)[..]
            .try_into()
            .expect("always split to 2");

        let [sidebar, main_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(37), Constraint::Fill(1)])
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
                rect.y += 1;
                rect
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let [mut navigation_list, bottom_sidebar] = Layout::default()
            .direction(Direction::Vertical)
            .flex(ratatui::layout::Flex::SpaceBetween)
            .constraints([Constraint::Max(10), Constraint::Fill(1)])
            .split(sidebar)[..]
            .try_into()
            .expect("split to 2");

        let [middle_sidebar, queue_list, state_badge] = Layout::default()
            .direction(Direction::Vertical)
            .flex(ratatui::layout::Flex::End)
            .constraints([
                Constraint::Fill(1),
                Constraint::Max(7),
                Constraint::Length(3),
            ])
            .split(bottom_sidebar)[..]
            .try_into()
            .expect("split to 2");

        // any idea how this can be used?
        navigation_list.height += middle_sidebar.height;

        let [main_area_up, main_area_down] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50); 2])
            .split(main_area)[..]
            .try_into()
            .expect("split into 2");

        let [main_bottom_left, main_bottom_right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50); 2])
            .split(main_area_down)[..]
            .try_into()
            .expect("split into 2");

        Dimension {
            window_border,
            overlay,

            searchbar: search_area,
            statusbar: (status_area, status_components),
            progressbar: bottom_area,
            queue_list,
            state_badge,
            navigation_list,
            music_pane: main_area_up,
            playlist_pane: main_bottom_left,
            artist_pane: main_bottom_right,
        }
    }
}
