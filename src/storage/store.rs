use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{
    Result,
    eyre::{WrapErr, eyre},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::models::{FailuresByStep, Language, MemorizingContext, MemorizingStep, Phrase};

use super::Settings;

const LANGUAGES_FILE_NAME: &str = "languages.json";
const PHRASES_FILE_NAME: &str = "phrases.json";
const SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Debug)]
pub struct Store {
    pub languages: Vec<Language>,
    pub phrases: Vec<Phrase>,
    pub settings: Settings,
    data_dir: PathBuf,
}

impl Store {
    pub fn load(data_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_dir)
            .wrap_err_with(|| format!("failed to create data directory {}", data_dir.display()))?;

        let languages = load_or_create(&data_dir.join(LANGUAGES_FILE_NAME), default_languages())?;
        let phrases = load_or_create(&data_dir.join(PHRASES_FILE_NAME), Vec::<Phrase>::new())?;
        let settings = load_or_create(&data_dir.join(SETTINGS_FILE_NAME), Settings::default())?;

        let store = Self {
            languages,
            phrases,
            settings,
            data_dir,
        };
        store.validate()?;
        Ok(store)
    }

    pub fn save_languages(&self) -> Result<()> {
        write_json(&self.data_dir.join(LANGUAGES_FILE_NAME), &self.languages)
    }

    pub fn save_phrases(&self) -> Result<()> {
        // TODO(optimization): записать это место как возможное место просадки производительности при большом объеме файлов
        write_json(&self.data_dir.join(PHRASES_FILE_NAME), &self.phrases)
    }

    pub fn save_settings(&self) -> Result<()> {
        self.validate_settings()?;
        write_json(&self.data_dir.join(SETTINGS_FILE_NAME), &self.settings)
    }

    pub fn update_settings(&mut self, settings: Settings) -> Result<()> {
        let previous_settings = std::mem::replace(&mut self.settings, settings);
        if let Err(error) = self.save_settings() {
            self.settings = previous_settings;
            return Err(error);
        }
        Ok(())
    }

    pub fn add_phrase(
        &mut self,
        original_language_id: u32,
        translation_language_id: u32,
        original_text: impl Into<String>,
        translation_text: impl Into<String>,
    ) -> Result<u32> {
        self.validate_language_id(original_language_id)?;
        self.validate_language_id(translation_language_id)?;

        let phrase = Phrase {
            id: self.next_phrase_id()?,
            original_language_id,
            translation_language_id,
            original_text: original_text.into(),
            translation_text: translation_text.into(),
            memorizing_context: MemorizingContext {
                current_step: MemorizingStep::New,
                needed_attempts: self.settings.needed_attempts,
                current_attempt: 0,
                last_attempt_time: None,
                failures: FailuresByStep::default(),
            },
        };
        let phrase_id = phrase.id;

        self.phrases.push(phrase);
        if let Err(error) = self.save_phrases() {
            self.phrases.pop();
            return Err(error);
        }

        Ok(phrase_id)
    }

    pub fn update_phrase(&mut self, phrase: Phrase) -> Result<()> {
        self.validate_phrase(&phrase)?;

        // TODO(optimization): записать это место как возможную просадку по производительности при большом количестве phrases.
        // Варианты для оптимизаций:
        // - разделить хранение phrases на блоки по MemorizingStep
        // - разделить уже разделенный по MemorizingStep блоки хранения по интервалам id или по hash map
        let index = self
            .phrases
            .iter()
            .position(|stored_phrase| stored_phrase.id == phrase.id)
            .ok_or_else(|| eyre!("phrase with id {} does not exist", phrase.id))?;

        let previous_phrase = std::mem::replace(&mut self.phrases[index], phrase);
        if let Err(error) = self.save_phrases() {
            self.phrases[index] = previous_phrase;
            return Err(error);
        }

        Ok(())
    }

    pub fn next_phrase_id(&self) -> Result<u32> {
        // TODO(optimization): записать это место как возможное для оптимизации при большом количестве phrases
        self.phrases
            .iter()
            .map(|phrase| phrase.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| eyre!("cannot allocate a new phrase id"))
    }

    fn validate(&self) -> Result<()> {
        self.validate_settings()?;
        for phrase in &self.phrases {
            self.validate_phrase(phrase)?;
        }
        Ok(())
    }

    fn validate_settings(&self) -> Result<()> {
        self.validate_language_id(self.settings.default_original_language_id)?;
        self.validate_language_id(self.settings.default_translation_language_id)?;
        if self.settings.needed_attempts == 0 {
            return Err(eyre!("needed_attempts must be greater than zero"));
        }
        if self.settings.reverse_translation_probability > 100 {
            return Err(eyre!(
                "reverse_translation_probability must be between 0 and 100"
            ));
        }
        Ok(())
    }

    fn validate_phrase(&self, phrase: &Phrase) -> Result<()> {
        self.validate_language_id(phrase.original_language_id)?;
        self.validate_language_id(phrase.translation_language_id)?;
        if phrase.memorizing_context.needed_attempts == 0 {
            return Err(eyre!(
                "phrase {} has needed_attempts equal to zero",
                phrase.id
            ));
        }
        Ok(())
    }

    fn validate_language_id(&self, language_id: u32) -> Result<()> {
        if self
            .languages
            .iter()
            .any(|language| language.id == language_id)
        {
            Ok(())
        } else {
            Err(eyre!("language with id {language_id} does not exist"))
        }
    }
}

fn default_languages() -> Vec<Language> {
    vec![
        Language {
            id: 1,
            name: "Russian".to_owned(),
        },
        Language {
            id: 2,
            name: "English".to_owned(),
        },
    ]
}

fn load_or_create<T>(path: &Path, default: T) -> Result<T>
where
    T: DeserializeOwned + Serialize,
{
    match fs::read_to_string(path) {
        Ok(contents) if !contents.trim().is_empty() => serde_json::from_str(&contents)
            .wrap_err_with(|| format!("failed to parse {}", path.display())),
        Ok(_) => {
            write_json(path, &default)?;
            Ok(default)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            write_json(path, &default)?;
            Ok(default)
        }
        Err(error) => Err(error).wrap_err_with(|| format!("failed to read {}", path.display())),
    }
}

fn write_json<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize + ?Sized,
{
    let json = serde_json::to_string_pretty(value)
        .wrap_err_with(|| format!("failed to serialize data for {}", path.display()))?;
    fs::write(path, format!("{json}\n"))
        .wrap_err_with(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::Store;

    static NEXT_TEMP_DIR_ID: AtomicU64 = AtomicU64::new(0);

    struct TempDataDir(PathBuf);

    impl TempDataDir {
        fn new() -> Self {
            let id = NEXT_TEMP_DIR_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "rdictionary-store-test-{}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("failed to create test directory");
            Self(path)
        }
    }

    impl Drop for TempDataDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn load_creates_default_files() {
        let data_dir = TempDataDir::new();

        let store = Store::load(data_dir.0.clone()).expect("failed to load store");

        assert_eq!(store.languages.len(), 2);
        assert!(store.phrases.is_empty());
        assert_eq!(store.settings.default_original_language_id, 1);
        assert!(data_dir.0.join("languages.json").is_file());
        assert!(data_dir.0.join("phrases.json").is_file());
        assert!(data_dir.0.join("settings.json").is_file());
    }

    #[test]
    fn add_and_update_phrase_are_persisted() {
        let data_dir = TempDataDir::new();
        let mut store = Store::load(data_dir.0.clone()).expect("failed to load store");

        let phrase_id = store
            .add_phrase(1, 2, "привет", "hello")
            .expect("failed to add phrase");
        assert_eq!(phrase_id, 1);

        let mut phrase = store.phrases[0].clone();
        phrase.translation_text = "hi".to_owned();
        store
            .update_phrase(phrase)
            .expect("failed to update phrase");

        let reloaded = Store::load(data_dir.0.clone()).expect("failed to reload store");
        assert_eq!(reloaded.phrases.len(), 1);
        assert_eq!(reloaded.phrases[0].translation_text, "hi");
    }
}
