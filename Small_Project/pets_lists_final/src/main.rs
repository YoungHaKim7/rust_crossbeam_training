use std::{
    fs, io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use serde::{Deserialize, Serialize};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};

const DB_PATH: &str = "./data/db.json";

#[derive(Serialize, Deserialize, Clone)]
struct Pet {
    id: usize,
    name: String,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Lists,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Lists => 1,
        }
    }
}

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "pet-CLI",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'l' to access the list, 'a' to add a random pet, 'd' to delete the selected pet, and 'm' to modify the selected pet.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}

fn read_db() -> Result<Vec<Pet>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

fn write_db(pets: &Vec<Pet>) -> Result<(), Error> {
    let db_content = serde_json::to_string(pets)?;
    fs::write(DB_PATH, db_content)?;
    Ok(())
}

// fn add_list() -> Result<(), Error> {
//     let mut rng = rand::thread_rng();
//     let mut pets = read_db()?;

//     let new_pet = Pet {
//         id: pets.len() + 1,
//         name: format!("Pet_{}", rng.gen::<u32>()),
//         category: if rng.gen_bool(0.5) { "Dog" } else { "Cat" }.to_string(),
//         age: rng.gen_range(1..15),
//         created_at: Utc::now(),
//     };

//     pets.push(new_pet);
//     write_db(&pets)?;
//     Ok(())
// }

fn add_list() -> Result<(), Error> {
    let mut pets = read_db()?;

    // Initialize input fields
    let mut name = String::new();
    let mut category = String::new();
    let mut age = String::new();
    let mut current_field = 0; // 0: Name, 1: Category, 2: Age

    // Function to render the form (moved outside the loop to avoid borrow issues)
    fn render_form<'a>(
        name: &'a str,
        category: &'a str,
        age: &'a str,
        current_field: usize,
    ) -> Paragraph<'a> {
        Paragraph::new(vec![
            Spans::from(vec![Span::raw("Enter pet details:")]),
            Spans::from(vec![
                Span::raw("Name: "),
                Span::styled(
                    name,
                    if current_field == 0 {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Spans::from(vec![
                Span::raw("Category: "),
                Span::styled(
                    category,
                    if current_field == 1 {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Spans::from(vec![
                Span::raw("Age: "),
                Span::styled(
                    age,
                    if current_field == 2 {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Spans::from(vec![Span::raw("Press Enter to submit, Esc to cancel.")]),
        ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Add Pet")
                .border_type(BorderType::Plain),
        )
    }

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let form = render_form(&name, &category, &age, current_field);
            rect.render_widget(form, chunks[0]);
        })?;

        if let CEvent::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Char(c) => match current_field {
                    0 => name.push(c),
                    1 => category.push(c),
                    2 => age.push(c),
                    _ => {}
                },
                KeyCode::Backspace => match current_field {
                    0 => {
                        name.pop();
                    }
                    1 => {
                        category.pop();
                    }
                    2 => {
                        age.pop();
                    }
                    _ => {}
                },
                KeyCode::Enter => {
                    if current_field < 2 {
                        current_field += 1; // Move to the next field
                    } else {
                        // Submit the form
                        let new_pet = Pet {
                            id: pets.len() + 1,
                            name: name.trim().to_string(),
                            category: category.trim().to_string(),
                            age: age.trim().parse().unwrap_or(1),
                            created_at: Utc::now(),
                        };
                        pets.push(new_pet);
                        write_db(&pets)?;
                        break;
                    }
                }
                KeyCode::Esc => {
                    break; // Cancel the form
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn delete_pet(id: usize) -> Result<(), Error> {
    let mut pets = read_db()?;
    pets.retain(|pet| pet.id != id);
    write_db(&pets)?;
    Ok(())
}

fn modify_pet(
    id: usize,
    new_name: String,
    new_category: String,
    new_age: usize,
) -> Result<(), Error> {
    let mut pets = read_db()?;
    if let Some(pet) = pets.iter_mut().find(|pet| pet.id == id) {
        pet.name = new_name;
        pet.category = new_category;
        pet.age = new_age;
    }
    write_db(&pets)?;
    Ok(())
}

fn render_pets<'a>(pet_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let pets = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Pets")
        .border_type(BorderType::Plain);

    let pet_list = read_db().expect("can fetch pet list");
    let items: Vec<_> = pet_list
        .iter()
        .map(|pet| {
            ListItem::new(Spans::from(vec![Span::styled(
                pet.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_pet = pet_list
        .get(
            pet_list_state
                .selected()
                .expect("there is always a selected pet"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(pets).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let pet_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_pet.id.to_string())),
        Cell::from(Span::raw(selected_pet.name)),
        Cell::from(Span::raw(selected_pet.category)),
        Cell::from(Span::raw(selected_pet.age.to_string())),
        Cell::from(Span::raw(selected_pet.created_at.to_string())),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Category",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Age",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
        Constraint::Percentage(20),
    ]);

    (list, pet_detail)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");
    let mut pet_list_state = ListState::default();
    pet_list_state.select(Some(0));

    let menu_titles = vec!["Home", "List", "Add", "Delete", "Modify", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let menu = menu_titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let tabs = Tabs::new(menu)
        .select(active_menu_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"));

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            rect.render_widget(tabs.clone(), chunks[0]);

            match active_menu_item {
                MenuItem::Home => {
                    let home = render_home();
                    rect.render_widget(home, chunks[1]);
                }
                MenuItem::Lists => {
                    let (list, table) = render_pets(&pet_list_state);
                    rect.render_widget(list, chunks[1]);
                    rect.render_widget(table, chunks[2]);
                }
            }

            let copyright = Paragraph::new("pet-CLI 2020 - all rights reserved")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );
            rect.render_widget(copyright, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('l') => active_menu_item = MenuItem::Lists,
                KeyCode::Char('a') => {
                    add_list().expect("can add new pet");
                    pet_list_state.select(Some(0));
                }
                KeyCode::Char('d') => {
                    let pets = read_db().expect("can fetch pet list");
                    if let Some(selected) = pet_list_state.selected() {
                        if selected < pets.len() {
                            delete_pet(pets[selected].id).expect("can delete pet");
                            pet_list_state.select(Some(0));
                        }
                    }
                }
                KeyCode::Char('m') => {
                    let pets = read_db().expect("can fetch pet list");
                    if let Some(selected) = pet_list_state.selected() {
                        if selected < pets.len() {
                            let pet = &pets[selected];
                            println!("Modify pet: {}", pet.name);
                            println!("Enter new name:");
                            let mut new_name = String::new();
                            io::stdin()
                                .read_line(&mut new_name)
                                .expect("can read input");
                            println!("Enter new category:");
                            let mut new_category = String::new();
                            io::stdin()
                                .read_line(&mut new_category)
                                .expect("can read input");
                            println!("Enter new age:");
                            let mut new_age = String::new();
                            io::stdin().read_line(&mut new_age).expect("can read input");
                            let new_age: usize = new_age.trim().parse().expect("valid age");

                            modify_pet(
                                pet.id,
                                new_name.trim().to_string(),
                                new_category.trim().to_string(),
                                new_age,
                            )
                            .expect("can modify pet");
                            pet_list_state.select(Some(0));
                        }
                    }
                }
                KeyCode::Up => {
                    let pets = read_db().expect("can fetch pet list");
                    if let Some(selected) = pet_list_state.selected() {
                        if selected > 0 {
                            pet_list_state.select(Some(selected - 1));
                        }
                    }
                }
                KeyCode::Down => {
                    let pets = read_db().expect("can fetch pet list");
                    if let Some(selected) = pet_list_state.selected() {
                        if selected < pets.len() - 1 {
                            pet_list_state.select(Some(selected + 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
