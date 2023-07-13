use mongodb::bson::oid::ObjectId;
use serenity::{framework::standard::CommandError, prelude::*};

use crate::{db, utils};

#[derive(
    Debug, serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum PdxGame {
    Eu4,
    Victoria3,
    Hoi4,
    Ck3,
    Stellaris,
    Aow4,
}

impl PdxGame {
    // MUST BE UPDATED WHEN CHANGING ENUM
    pub fn iterator() -> impl Iterator<Item = PdxGame> {
        [
            Self::Eu4,
            Self::Victoria3,
            Self::Hoi4,
            Self::Ck3,
            Self::Stellaris,
            Self::Aow4,
        ]
        .iter()
        .copied()
    }
}

impl std::fmt::Display for PdxGame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let s = match *self {
            PdxGame::Eu4 => "Europa Universalis IV",
            PdxGame::Victoria3 => "Victoria 3",
            PdxGame::Hoi4 => "Hearts of Iron IV",
            PdxGame::Ck3 => "Crusader Kings III",
            PdxGame::Stellaris => "Stellaris",
            PdxGame::Aow4 => "Age of Wonders 4",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct PdxFollow {
    _id: ObjectId,
    user_id: String,
    follows: Vec<(PdxGame, bool)>,
}

impl PdxFollow {
    pub fn new(user_id: String) -> Self {
        let follows = PdxGame::iterator().map(|g| (g, true)).collect();
        Self {
            _id: ObjectId::new(),
            user_id,
            follows,
        }
    }

    pub fn follows(&self) -> impl Iterator<Item = (PdxGame, bool)> + '_ {
        self.follows.iter().copied()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct GameLinks {
    game: PdxGame,
    latest: String,
    previous: String,
}

impl GameLinks {
    pub fn update(&mut self, new_link: String) -> &mut Self {
        self.previous = self.latest.clone();
        self.latest = new_link;
        self
    }

    pub fn embed_value(&self) -> (String, String) {
        (
            self.game.to_string(),
            format!("[Dernier]({})\n[Précédent]({})", self.latest, self.previous),
        )
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct PdxLinks {
    _id: ObjectId,
    games: Vec<GameLinks>,
}

impl PdxLinks {
    pub fn game_links(&self, game: PdxGame) -> Option<GameLinks> {
        self.games.iter().find(|gl| gl.game == game).cloned()
    }

    pub fn all_latest(&self) -> Vec<(PdxGame, String)> {
        self.games
            .clone()
            .into_iter()
            .map(|gl| (gl.game, gl.latest))
            .collect()
    }

    #[allow(dead_code)]
    pub fn all_previous(&self) -> Vec<(PdxGame, String)> {
        self.games
            .clone()
            .into_iter()
            .map(|gl| (gl.game, gl.previous))
            .collect()
    }

    pub async fn db_links(ctx: &Context) -> Result<Self, CommandError> {
        db::find_filter(ctx, "pdx_links", None)
            .await?
            .ok_or(utils::command_error("no pdx link db"))
    }

    pub fn update(&mut self, game: PdxGame, link: Option<String>) -> Result<(), String> {
        if let Some(new_link) = link {
            if let Some(gl) = self.games.iter_mut().find(|gl| gl.game == game) {
                gl.update(new_link);
                Ok(())
            } else {
                Err(format!("Le jeu {game:?} ne fait pas partie du PdxLinks"))
            }
        } else {
            Ok(())
        }
    }

    // reset function
    #[allow(dead_code)]
    pub fn init() -> Self {
        let games = vec![
            GameLinks {game: PdxGame::Eu4, latest: "https://forum.paradoxplaza.com/forum/developer-diary/europa-universalis-iv-development-diary-23rd-of-may-2023-1-35-3-known-issues-and-the-road-to-1-35-4.1586331/".to_owned(), previous: "https://forum.paradoxplaza.com/forum/developer-diary/europa-universalis-iv-development-diary-25th-of-april-2023-1-35-post-release-support.1579720/".to_owned()},
            GameLinks {game: PdxGame::Victoria3, latest: "https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-89-whats-next-after-1-3.1589178/".to_owned(), previous: "https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-88-voice-of-the-people-narrative-content-improvements.1588003/".to_owned()},
            GameLinks {game: PdxGame::Hoi4, latest: "https://forum.paradoxplaza.com/forum/developer-diary/developer-diary-historical-norway.1590854/".to_owned(), previous: "https://forum.paradoxplaza.com/forum/developer-diary/developer-diary-historical-sweden.1589418/".to_owned()},
            GameLinks {game: PdxGame::Ck3, latest: "https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-130-wards-and-wardens-the-vision.1590033/".to_owned(), previous: "https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-129-post-release-update-extra-content.1586430/".to_owned()},
            GameLinks {game: PdxGame::Stellaris, latest: "https://forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-304-3-8-4-released-whats-next.1589870/".to_owned(), previous: "https://forum.paradoxplaza.com/forum/developer-diary/stellaris-dev-diary-303-stellaris-with-a-twist-community-event.1587986/".to_owned()},
            GameLinks {game: PdxGame::Aow4, latest: "https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-18-dragon-lords.1589296/".to_owned(), previous: "https://forum.paradoxplaza.com/forum/developer-diary/dev-diary-17-post-launch.1587996/".to_owned()}
        ];
        Self {
            _id: ObjectId::new(),
            games,
        }
    }
}
