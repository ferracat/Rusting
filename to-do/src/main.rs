use ncurses::*;
use std::cmp::min;

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;
const TITLE_PAIR: i16 = 2;

type Id = usize;

#[derive(Default)]
struct Ui {
    list_curr: Option<Id>,
    row: usize,
    col: usize,
}

impl Ui {
    fn begin(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }

    fn end(&mut self) {}

    fn begin_list(&mut self, id: Id) {
        assert!(self.list_curr.is_none(), "Nested lists are not allowed!");
        self.list_curr = Some(id)
    }

    fn end_list(&mut self) {
        self.list_curr = None;
    }

    fn list_element(&mut self, label: &str, id: Id) {
        let id_curr = self.list_curr.expect(&format!(
            "Not allowed to create list elements outside of lists"
        ));

        let pair = {
            if id_curr == id {
                HIGHLIGHT_PAIR
            } else {
                REGULAR_PAIR
            }
        };

        self.label(&format!("- [ ] {}", label), pair);
    }

    fn label(&mut self, text: &str, pair: i16) {
        mv(self.row as i32, self.col as i32);
        attron(COLOR_PAIR(pair));
        addstr(text);
        attroff(COLOR_PAIR(pair));
        self.row += 1;
    }
}

// ------------------------------------------------------------------------------------------------
//                                             MAIN
// ------------------------------------------------------------------------------------------------
fn main() {
    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);
    init_pair(TITLE_PAIR, COLOR_YELLOW, COLOR_BLACK);

    let todos: Vec<String> = vec![
        "Write the todo app".to_string(),
        "Buy a bread".to_string(),
        "Make a cup of tea".to_string(),
    ];
    let dones: Vec<String> = vec![
        "Start the stream".to_string(),
        "Have a breakfast".to_string(),
        "Make a cup of tea".to_string(),
    ];
    let mut todo_curr: usize = 0;
    let mut done_curr: usize = 0;
    let mut quit = false;

    let mut ui = Ui::default();

    while !quit {
        ui.begin(0, 0);
        {
            ui.label(">>> TODO:", TITLE_PAIR);
            ui.begin_list(todo_curr);
            for (index, todo) in todos.iter().enumerate() {
                ui.list_element(todo, index);
            }
            ui.end_list();

            ui.label("--------------------------------", REGULAR_PAIR);

            ui.label(">>> DONE:", TITLE_PAIR);
            ui.begin_list(done_curr + 6969);
            for (index, done) in dones.iter().enumerate() {
                ui.list_element(done, index + 6969);
            }
            ui.end_list();
        }
        ui.end();

        refresh();

        let key = getch();
        match key as u8 as char {
            'q' => quit = true,
            /*KEY_UP*/
            'w' => {
                if todo_curr > 0 {
                    todo_curr -= 1;
                }
            }
            /*KEY_DOWN*/
            's' => todo_curr = min(todo_curr + 1, todos.len() - 1),
            _ => {}
        }
    }

    endwin();
}
