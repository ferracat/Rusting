//use ncurses::*;
use ncursesw::shims::bindings::chtype;
use ncursesw::*;
use std::thread;
use std::time::Duration;


fn main() {
    let mut origem = Origin {
        x = 0,
        y = 0,
    }
    
    // Initialize ncurses
    initscr();
    cbreak();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    keypad(stdscr(), true);

    
    // Progress bar parameters
    let total_width = 10;

    let bars_chars = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

    for progress in 0..=total_width
    {
        let percentage = (progress as f32 / total_width as f32 * 100.0) as i32;
        
        // Calculate the number of filled and empty blocks
        let filled_blocks = (progress as f32 / total_width as f32 * total_width as f32) as i32;
        let empty_blocks = total_width - filled_blocks;

        // Print the progress bar
        mvadd_wch(origem, '[' as cchar_t);
        for _ in 0..filled_blocks {
            //mvaddch(0, getcurx(stdscr()), ACS_CKBOARD());
            origem.x = getcurx(stdscr());
            origem.y = 0;
            mvadd_wch(origem, 'X' as u32);
        }
        for _ in 0..empty_blocks {
            origem.x = getcurx(stdscr());
            origem.y = 0;
            mvadd_wch(origem, ' ' as cchar_t);
        }
        mvadd_wch({0, getcurx(stdscr())}, ']' as cchar_t);

        // Print the percentage
        mvprintw(1, 0, &format!("{}%% complete         ", percentage));
        refresh();

        // Sleep for a short duration
        thread::sleep(Duration::from_millis(100));
    }

    origem.x = 0;
    origem.y = 3;
    mvadd_wch(origem, ' ' as cchar_t);
    for c in bars_chars
    {
        origem.x = getcurx(stdscr());
        origem.y = 3;
        mvadd_wch(origem, c as cchar_t);

        // Sleep for a short duration
        thread::sleep(Duration::from_millis(1000));
    }
    //println!("▏");
    
    // Wait for user input
    getch();

    // Clean up ncurses
    endwin();    
}
