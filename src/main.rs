use std::{env, fs, process, thread};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crossterm::terminal::{self, ClearType};


use std::{error::Error, io};
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::slice::SliceIndex;
use std::thread::sleep;
use std::time::{Duration, Instant};
#[cfg(feature = "ratatui")]
use ratatui as tui;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Corner},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs,List, ListItem, ListState},
    Frame, Terminal,
};
use tui::layout::{Alignment, Rect};
use tui::widgets::{Paragraph, StatefulWidget, Wrap};
use std::io::stdout;
use std::ptr::null;
use std::sync::{Arc, Mutex};
use throbber_widgets_tui::ThrobberState;
use tui::buffer::Buffer;
use tui::text::Text;


struct GlobalState {
    level: usize,
    current_tab: usize,
}


impl GlobalState {
    fn new() -> Self {
        GlobalState {
            level: 0,
            current_tab: 0,
        }
    }
}


#[derive(Clone)]
struct InnerListState {
    selected: Option<usize>,
}

impl InnerListState {
    fn default() -> Self {
        InnerListState {
            selected: None,
        }
    }
}

#[derive()]
struct MyThrobber<'l> (throbber_widgets_tui::Throbber<'l>);
#[derive(Clone)]
struct ThrobberStatea {
    start_time: Instant,
}

impl ThrobberStatea {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}


struct Throbber {
    state: ThrobberStatea,
}

impl StatefulWidget for Throbber {
    type State = ();

    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer, state: &mut Self::State) {
        let elapsed_ms = self.state.elapsed_ms();
        let frames = ["-", "\\", "|","|", "/"];
        let frame = frames[((elapsed_ms / 100) % frames.len() as u64) as usize];
        let text = format!("Processing... {}", frame);
        buf.set_string(area.x, area.y, text, tui::style::Style::default());
    }

}

fn render_throbber<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    state: &mut ThrobberStatea,
) {
    let throbber = Throbber { state: state.clone() };
    f.render_stateful_widget(throbber, area, &mut ());
}

/*
impl StatefulWidget for MyThrobber {
    type State = ThrobberState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let throbber = throbber_widgets_tui::Throbber::default();
        let full = MyThrobber(throbber_widgets_tui::Throbber::default()
            .label("Running...")
            .throbber_set(throbber_widgets_tui::BRAILLE_EIGHT_DOUBLE)
            .use_type(throbber_widgets_tui::WhichUse::Spin));
        // Assume chunks[3] is where you want to render the throbber:
        f.render_stateful_widget(full, chunks[3], &mut self.throbber_state);
        // ... rendering code for the throbber
        throbber.render(area, buf);

    }
}*/

#[derive(Clone)]
struct StatefulList {
    state: ListState,
    inner_states: Vec<InnerListState>,
    items: Vec<Vec<( String, i32)>>,
    level: usize,  // 0 for outer list, 1 for inner list
    inner_index: usize,  // Index in the inner list
}

impl  StatefulList {
    fn with_items(items: Vec<Vec<( String, i32)>>) -> StatefulList {
        let inner_states = vec![InnerListState::default(); items.len()];

        StatefulList{
            state: ListState::default(),
            inner_states,
            items,
            level : 0,
            inner_index :0,
        }
    }
    pub fn next_item(&mut self, rc: Rc<RefCell<GlobalState>>) {
        let state = rc.borrow();
        let tab = state.current_tab;
        let inner_state = &mut self.inner_states[tab];
        inner_state.selected = Some((inner_state.selected.unwrap_or(0) + 1) % self.items[tab].len());
    }


    pub fn previous_item(&mut self, rc: Rc<RefCell<GlobalState>>) {
        {
            let tab = rc.borrow().current_tab;

            let inner_state = &mut self.inner_states[tab];
            inner_state.selected = Some(
                if self.items[tab].is_empty() {
                    0
                } else {
                    inner_state
                        .selected
                        .and_then(|selected| selected.checked_sub(1))
                        .unwrap_or_else(|| self.items[tab].len() - 1)
                }
            );
        }
    }




    fn unselect(&mut self) {
        self.state.select(None);
        for inner_state in &mut self.inner_states {
            inner_state.selected = None;
        }
    }

}

#[derive(Clone)]

struct App<'a>{
    pub titles: Vec<&'a str>,
    pub index: usize,
    items: StatefulList,
    events: Vec<(&'a str, &'a str)>,
    output: Arc<Mutex<String>>,
    command_running: bool,
    clear_output: bool,
    command_running_icon: Arc<Mutex<bool>>,
    throbber_state: throbber_widgets_tui::ThrobberState,
}

struct StatefulThrobber<'a> {
    throbber: throbber_widgets_tui::Throbber<'a>,
    state: throbber_widgets_tui::ThrobberState,
}

impl<'a> App <'a> {
    fn new() -> App<'a>{
        App{
            titles : vec!["exec","checks","cleanup","config", "ios","android", "exit"],
            index:0,
            items: StatefulList::with_items(vec![
                vec![("flutter run".parse().unwrap(), 1),
                     ("flutter pub get".parse().unwrap(), 1),
                     ("flutter channel".parse().unwrap(), 1),],
                vec![ ("flutter clean".parse().unwrap(), 2),
                      ("flutter build".parse().unwrap(), 2),
                      ("flutter doctor".parse().unwrap(), 2),],
                vec![  ( "flutter clean cache".parse().unwrap(), 3),
                       ( "flutter repair".parse().unwrap(), 3),
                       ("flutter remove cache".parse().unwrap(), 3) ],
                vec![  ( "flutter devices".parse().unwrap(), 4),
                       ( "flutter logs".parse().unwrap(), 4),
                       ( "flutter emulators".parse().unwrap(), 4),
                ],
                vec![  ( "flutter install".parse().unwrap(), 5),
                       ( "flutter pod clean up".parse().unwrap(), 5),
                       ( "flutter deintegrate".parse().unwrap(), 5),
                       ( "flutter repo update".parse().unwrap(), 5),
                ],
                vec![  ( "flutter devices".parse().unwrap(), 6),
                       ( "flutter logs".parse().unwrap(),  6),
                       ( "flutter emulators".parse().unwrap(), 6),
                ],
                vec![  ( "exit".parse().unwrap(), 6)],

            ]),
            events: vec![
            ],
            output: Arc::new(Mutex::new("".parse().unwrap())),
            command_running: false,
            clear_output: false,
            command_running_icon: Arc::new(Mutex::new(false)),
            throbber_state: throbber_widgets_tui::ThrobberState::default()
        }
    }

    fn run_command(&self, cmd_arg1:String, cmd_arg2: String, errorMsg: Option<String>) {
        let command_running = self.command_running_icon.clone();
        let output = self.output.clone();  // Assuming `self.output` is of type Arc<Mutex<String>>
        thread::spawn(move || {
            *command_running.lock().unwrap() = true;
            let output_result = Command::new(&cmd_arg1).arg(&cmd_arg2).output()
                .expect("Failed to execute command");
            let mut shared_output = output.lock().unwrap();
            *shared_output = String::from_utf8_lossy(&output_result.stdout).into_owned();
            *command_running.lock().unwrap() = false;
        });
    }





    pub fn next(&mut self, rc: Rc<RefCell<GlobalState>>){
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self, rc: Rc<RefCell<GlobalState>>){
        if self.index > 0{
            self.index -= 1;
        }else{
            self.index = self.titles.len() - 1;
        }
    }
    pub fn enter(&mut self, mut app: App, rc: Rc<RefCell<GlobalState>> ) {
        self.command_running = true;

        app.clear_output = true;

        if app.clear_output {
            execute!(
            stdout(),
            terminal::Clear(ClearType::All)
        ).unwrap();
            app.clear_output = false;
            let mut output = app.output.lock().unwrap();
            output.clear();
        }

        let inner_state = &self.items.inner_states[self.index];

        if let Some(inner_selected) = inner_state.selected {
            match self.index {
                0 => {
                    match inner_selected {
                        0 => {
                            self.run_command("flutter".parse().unwrap(), "run".parse().unwrap(), None);
                        },
                        1 => {
                            self.run_command("flutter".parse().unwrap(), "pub get".parse().unwrap(), None);
                        },
                        2 => {
                            // may get rid of this
                            self.run_command("flutter".parse().unwrap(), "channel stable".parse().unwrap(), None);
                        },
                        2 => {
                            // may get rid of this
                            self.run_command("flutter".parse().unwrap(), "create test".parse().unwrap(), None);
                        },

                        _ => {

                        }
                    }
                },
                1 => {
                    match inner_selected {
                        0 => {

                            self.run_command("flutter".parse().unwrap(), "clean".parse().unwrap(), None);
                        },
                        1 => {
                            if is_ios_dir() {
                                self.run_command("flutter".parse().unwrap(), "build ios".parse().unwrap(), None);
                            }
                        },
                        2 => {
                            // Command::new("flutter").arg("doctor").output().expect("Failed to execute command");
                            // let output = Command::new("flutter").arg("doctor").output()
                            //     .expect("Failed to execute command");
                            self.run_command("flutter".parse().unwrap(), "doctor".parse().unwrap(), None);

                            // self.output = String::from_utf8_lossy(&output.stdout).into_owned();
                        },
                        _ => {}
                    }
                },
                2 => {
                    match inner_selected {
                        0 => {
                            self.run_command("flutter".parse().unwrap(), "clean".parse().unwrap(), None);
                        },
                        1 => {
                            self.run_command("flutter".parse().unwrap(), "build".parse().unwrap(), None);
                        },
                        2 => {
                            self.run_command("flutter".parse().unwrap(), "doctor".parse().unwrap(), None);
                        },
                        _ => {}
                    }
                },
                3 => {
                    match inner_selected {
                        0 => {
                            self.run_command("flutter".parse().unwrap(), "pub cache clean".parse().unwrap(), None);
                        },
                        1 => {
                            self.run_command("flutter".parse().unwrap(), "cache repair".parse().unwrap(), None);
                        },
                        2 => {
                            self.run_command("rm".parse().unwrap(), "~/flutter/.pub-cache".parse().unwrap(), None);
                        },
                        _ => {}
                    }
                },
                4 => {
                    match inner_selected {
                        0 => {
                            if is_ios_dir() {
                                self.run_command("flutter".parse().unwrap(), "build ipa".parse().unwrap(), None);
                            }
                        },
                        1 => {
                            if is_ios_dir() {
                                self.run_command("flutter".parse().unwrap(), "clean && cd ios && rm Podfile Podfile.lock && rm -rf Pods/ && cd .. && flutter pub get".parse().unwrap(), None);
                            }
                        },
                        2 => {
                            if is_ios_dir() {
                                self.run_command("cd".parse().unwrap(), "ios".parse().unwrap(), None);
                                self.run_command("flutter".parse().unwrap(), "deintegrate".parse().unwrap(), None);
                            }
                        },
                        3 => {
                            // Command::new("flutter").arg("doctor").output().expect("Failed to execute command");
                            // check is this is ios directory
                            if is_ios_dir() {
                                self.run_command("cd".parse().unwrap(), "ios".parse().unwrap(), None);

                                self.run_command("flutter".parse().unwrap(), "repo-update".parse().unwrap(), None);
                            }
                            //self.output = String::from_utf8_lossy(&output.stdout).into_owned();
                        },
                        4 => {
                            if is_ios_dir() {
                                self.run_command("cd".parse().unwrap(), "ios".parse().unwrap(), None);

                                self.run_command("flutter".parse().unwrap(), "clean".parse().unwrap(), None);
                            }
                        },
                        _ => {}
                    }
                },
                5 => {
                    match inner_selected {
                        0 => {
                            self.run_command("flutter".parse().unwrap() , "clean".parse().unwrap(), None);
                        },
                        1 => {
                            self.run_command("flutter".parse().unwrap() , "build ios".parse().unwrap(), None);
                        },
                        2 => {
                            self.run_command("flutter".parse().unwrap() , "doctor".parse().unwrap(), None);
                        },
                        _ => {}
                    }
                },
                6 => {
                    match inner_selected {
                        0=> {process::exit(0);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        self.command_running = false;


    }

}

fn is_ios_dir() -> bool {
    let  is_flutter_project_dir  = fs::metadata("Podfile").is_ok() && fs::metadata("ios/").is_ok();
    if is_flutter_project_dir {
        println!("You are in a Flutter project directory.");
        return true;
    } else {
        println!("You are NOT in a Flutter project directory.");
    }
    false
}


fn is_flutter_dir(){
    let  is_flutter_project_dir  = fs::metadata("pubspec.yaml").is_ok() && fs::metadata("lib/").is_ok();
    if is_flutter_project_dir {
        println!("You are in a Flutter project directory.");
    } else {
        println!("You are NOT in a Flutter project directory.");
    }
}

fn main() -> Result<(), Box<dyn Error>> {

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let global_state = Rc::new(RefCell::new(GlobalState::new()));
    let res = run_app(&mut terminal, app, global_state );

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }
    env::set_var("RUST_BACKTRACE", "1");
    Ok(())
}


fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App,  global_state: Rc<RefCell<GlobalState>>
) -> io::Result<()> {
    if app.command_running {
        return Ok(());
    }
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Right => {
                    let mut state = global_state.borrow_mut();
                    state.current_tab = (state.current_tab + 1) % app.titles.len();
                    app.next(global_state.clone());
                },
                KeyCode::Left => {
                    let mut state = global_state.borrow_mut();
                    state.current_tab = if state.current_tab == 0 {
                        app.titles.len() - 1
                    } else {
                        state.current_tab - 1
                    };
                    app.previous(global_state.clone());
                },
                KeyCode::Enter => app.enter(app.clone(), global_state.clone()),
                KeyCode::Down =>   {
                    {
                        let mut state = global_state.borrow_mut();
                        let level = state.level;
                        let end = app.items.items[level].len() - 1;
                        if level == end{
                            state.level = end;
                        }else{
                            state.level =  state.level  + 1 ;
                        }
                    }
                    app.items.next_item(global_state.clone());
                },
                KeyCode::Up => {
                    let new_level = {
                        let state = global_state.borrow();
                        if state.level <= 0 { 0 } else { state.level - 1 }
                    };
                    {
                        let mut state = global_state.borrow_mut();
                        state.level = new_level;
                    }
                    app.items.previous_item(global_state.clone());
                },
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut  App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([
            Constraint::Length(3),  // for the tabs
            Constraint::Length(3),
            Constraint::Percentage(48),  // for the list and inner block
            Constraint::Percentage(48),  // for the output
        ].as_ref())
        .split(size);


    let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::Cyan));
    f.render_widget(block, size);

    let output_block = Block::default()
        .borders(Borders::ALL).title_alignment(Alignment::Center)
        .title("Output");

    let size = f.size();
    let width_per_tab = size.width / app.titles.len() as u16;

    let titles = app
        .titles
        .iter()
        .map(|t| {
            let text_width = t.len() as u16;
            let padding = (width_per_tab - text_width) / 3;
            let padded_title = format!("{:padding$}{}{:padding$}", "", t, "", padding = padding as usize);
            Spans::from(vec![
                Span::styled(padded_title, Style::default().fg(Color::Magenta)),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Flutter Command Center").title_alignment(Alignment::Center))
        .select(app.index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, chunks[0]);

    let items: Vec<ListItem> = {
        if let Some(inner_list) = app.items.items.get(app.index) {
            let inner_state = &app.items.inner_states[app.index];

            inner_list.iter().enumerate().map(|(idx, (text, number))| {
                let style = if Some(idx) == inner_state.selected {
                    Style::default().fg(Color::Black).bg(Color::White)
                } else {
                    Style::default()
                };
                let span = Span::styled(format!("{}: {}", text, number), style);
                ListItem::new(Spans::from(vec![span]))
            }).collect()
        } else {
            Vec::new() // No matching inner list for this tab, no items to show
        }
    };
    // render the current working directory box
    let cwd = env::current_dir().unwrap().display().to_string();
    let cwd_block = Block::default()
        .title("Current Working Directory")
        .borders(Borders::ALL);
    let cwd_paragraph = Paragraph::new(cwd)
        .block(cwd_block);
    f.render_widget(cwd_paragraph, chunks[1]);

    let inner = match app.index {
        0 => Block::default().title("Execution").borders(Borders::ALL).title_alignment(Alignment::Center),
        1 => Block::default().title("Health Checks").borders(Borders::ALL).title_alignment(Alignment::Center),
        2 => Block::default().title("Clean Up").borders(Borders::ALL).title_alignment(Alignment::Center),
        3 => Block::default().title("Flutter Config").borders(Borders::ALL).title_alignment(Alignment::Center),
        4 => Block::default().title("Ios Commands").borders(Borders::ALL).title_alignment(Alignment::Center),
        5 => Block::default().title("Android Commands").borders(Borders::ALL).title_alignment(Alignment::Center),
        6 => Block::default().title("Exit").borders(Borders::ALL).title_alignment(Alignment::Center),
        _ => unreachable!(),
    };
    f.render_widget(inner, chunks[2]);


    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, chunks[2], &mut app.items.state);
    let output = app.output.lock().unwrap();
    let output_text = Text::from(output.as_str());
    let output_paragraph = Paragraph::new(output_text)
        .block(output_block)
        .wrap(Wrap { trim: true });

    f.render_widget(output_paragraph, chunks[3]);  // Render in the third chunk
    let mut throbber_state = ThrobberStatea::new();


    if *app.command_running_icon.lock().unwrap() {
        render_throbber(f, chunks[3], &mut throbber_state);
    }
}
