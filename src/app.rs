//use crate::api::get_test::get_test;
use crate::api::utils::collect_personalized_view::*;
use crate::api::utils::collect_get_all_books::*;
use crate::api::libraries::get_library_perso_view::*;
use crate::api::libraries::get_all_books::*;
use crate::api::server::auth::*;
use crate::logic::handle_input::handle_l::*;
use crate::config::load_config;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    widgets::{ListState},
    DefaultTerminal,
};

// tui-textarea
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tui_textarea::TextArea;
use tui_textarea::Input;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{BLUE, SLATE},
        Color, Modifier, Style, Stylize,
    },
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem , Paragraph, StatefulWidget,
        Widget,
    },
};
use tui_textarea::Key;

pub enum AppView {
    Home,
    Library,
    SearchBook,
}

pub struct App {
   pub view_state: AppView,
   pub should_exit: bool,
   pub token: Option<String>,
   pub list_state_cnt_list: ListState,
   pub list_state_library: ListState,
   pub list_state_search_book: ListState,
   pub titles_cnt_list: Vec<String>,
   pub auth_names_cnt_list: Vec<String>,
   pub ids_cnt_list: Vec<String>,
   pub titles_library: Vec<String>,
   pub ids_library: Vec<String>,
   pub auth_names_library: Vec<String>,
   pub ids_search_book: Vec<String>,
   pub search_query: String,
   pub search_mode: bool,
}

/// Init app
 impl App {
     pub async fn new() -> Result<Self> {
         let config = load_config()?;
         let token =
             login(&config.credentials.id.to_string(), &config.credentials.password.to_string())
             .await?;

         // init for `Continue Listening`
         let continue_listening = get_continue_listening(&token).await?;
         let titles_cnt_list = collect_titles_cnt_list(&continue_listening).await;
         let auth_names_cnt_list = collect_auth_names_cnt_list(&continue_listening).await;
         let ids_cnt_list = collect_ids_cnt_list(&continue_listening).await;

         //init for `Library ` (all books of a shelf)
         let all_books = get_all_books(&token).await?;
         let titles_library = collect_titles_library(&all_books).await;
         let ids_library = collect_ids_library(&all_books).await;
         let auth_names_library = collect_auth_names_library(&all_books).await;

         // init for `Search Book`
         let ids_search_book: Vec<String> = Vec::new();
         let search_mode = false;
         let search_query = "  ".to_string();

         let view_state = AppView::Home; // By default, Home will be the first AppView launched
                                         // when the app start

         // Init ListeState for `continue Listening` list
         let mut list_state_cnt_list = ListState::default(); // init the ListState ratatui's widget
         list_state_cnt_list.select(Some(0)); // select the first item of the list when app is launch

         // Init ListeState for `Library` list
         let mut list_state_library = ListState::default(); // init the ListState ratatui's widget
         list_state_library.select(Some(0)); // select the first item of the list when app is launch
                                             
         // Init ListeState for `titles_search_book` list
         let mut list_state_search_book = ListState::default(); // init the ListState ratatui's widget
         list_state_search_book.select(Some(0)); // select the first item of the list when app is launch

        Ok(Self {
            should_exit: false,
            token: Some(token),
            list_state_cnt_list,
            list_state_library,
            list_state_search_book,
            titles_cnt_list,
            auth_names_cnt_list,
            ids_cnt_list,
            view_state,
            titles_library,
            ids_library,
            auth_names_library,
            ids_search_book,
            search_mode,
            search_query,
        })
    }

   /// handle events
   pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        Ok(())
    }

   /// handle key
    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char('s') => { self.search_activ();}
            KeyCode::Tab => self.toggle_view(),
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                // clone needed because variables will be use in an spawn
                let token = self.token.clone();
                let port = "1234".to_string();

                // init for  `Contnue Listening`
                let ids_cnt_list = self.ids_cnt_list.clone();
                let selected_cnt_list = self.list_state_cnt_list.selected();

                // init for `Library`
                let ids_library = self.ids_library.clone();
                let selected_library = self.list_state_library.selected();

                // init for `Search Book`
                let ids_search_book = self.ids_search_book.clone();
                let selected_search_book = self.list_state_search_book.selected();

                // Now, spawn the async task based on the current view state
                match self.view_state {
                    AppView::Home => {
                        tokio::spawn(async move {
                            handle_l(token.as_ref(), ids_cnt_list, selected_cnt_list, port).await;
                        });
                    }
                    AppView::Library => {
                        tokio::spawn(async move {
                            handle_l(token.as_ref(), ids_library, selected_library, port).await;
                        });
                    }
                    AppView::SearchBook => {
                        tokio::spawn(async move {
                            handle_l(token.as_ref(), ids_search_book, selected_search_book, port).await;
                        });
                    }
                }
            }
            _ => {}
        }
    }

    /// Toggle between Home and Library views
    fn toggle_view(&mut self) {
        self.view_state = match self.view_state {
            AppView::Home => AppView::Library,
            AppView::Library => AppView::Home,
            AppView::SearchBook => AppView::Home,
        };
    }

    /// Select functions that apply to both views
    /// all select functions are from ListState widget
    pub fn select_next(&mut self) {
        match self.view_state {
            AppView::Home => self.list_state_cnt_list.select_next(),
            AppView::Library => self.list_state_library.select_next(),
            AppView::SearchBook => self.list_state_search_book.select_next(),
        }
    }

    pub fn select_previous(&mut self) {
        match self.view_state {
            AppView::Home => self.list_state_cnt_list.select_previous(),
            AppView::Library => self.list_state_library.select_previous(),
            AppView::SearchBook => self.list_state_search_book.select_previous(),
        }
    }

    pub fn select_first(&mut self) {
        match self.view_state {
            AppView::Home => self.list_state_cnt_list.select_first(),
            AppView::Library => self.list_state_library.select_first(),
            AppView::SearchBook => self.list_state_search_book.select_first(),
        }
    }

    pub fn select_last(&mut self) {
        match self.view_state {
            AppView::Home => self.list_state_cnt_list.select_last(),
            AppView::Library => self.list_state_library.select_last(),
            AppView::SearchBook => self.list_state_search_book.select_last(),
        }
    }

 }
