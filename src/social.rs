use crate::error::{CliptionsError, Result};
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

/// Tweet ID extracted from URLs
pub type TweetId = String;

/// Represents announcement data for social media
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementData {
    pub block_num: u64,
    pub state_name: String,
    pub target_time: String,
    pub hashtags: Vec<String>,
    pub message: String,
    pub prize_pool: Option<f64>,
    pub livestream_url: Option<String>, // Add this field
}

/// Represents a social media task execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_name: String,
    pub parameters: HashMap<String, String>,
    pub timeout_seconds: u32,
}

/// Social media task interface
pub trait SocialTask {
    /// Execute the social media task
    fn execute(&self, context: &TaskContext) -> Result<String>;

    /// Get task name
    fn get_name(&self) -> &str;

    /// Validate task parameters
    fn validate_parameters(&self, params: &HashMap<String, String>) -> Result<()>;
}

/// URL parser for social media platforms
pub struct UrlParser {
    twitter_regex: Regex,
}

impl UrlParser {
    /// Create a new URL parser
    pub fn new() -> Result<Self> {
        let twitter_regex =
            Regex::new(r"https?://(?:www\.)?(?:twitter\.com|x\.com)/[^/]+/status/(\d+)")
                .map_err(|e| CliptionsError::ValidationError(format!("Invalid regex: {}", e)))?;

        Ok(Self { twitter_regex })
    }

    /// Extract tweet ID from Twitter/X URL
    pub fn extract_tweet_id(&self, url: &str) -> Result<TweetId> {
        if let Some(captures) = self.twitter_regex.captures(url) {
            if let Some(tweet_id) = captures.get(1) {
                return Ok(tweet_id.as_str().to_string());
            }
        }

        Err(CliptionsError::ValidationError(format!(
            "Invalid Twitter URL: {}",
            url
        )))
    }

    /// Validate URL format
    pub fn validate_url(&self, url: &str) -> Result<()> {
        Url::parse(url)
            .map_err(|e| CliptionsError::ValidationError(format!("Invalid URL: {}", e)))?;
        Ok(())
    }

    /// Extract domain from URL
    pub fn extract_domain(&self, url: &str) -> Result<String> {
        let parsed = Url::parse(url)
            .map_err(|e| CliptionsError::ValidationError(format!("Invalid URL: {}", e)))?;

        Ok(parsed.domain().unwrap_or("unknown").to_string())
    }
}

impl Default for UrlParser {
    fn default() -> Self {
        Self::new().expect("Failed to create URL parser")
    }
}

/// Hashtag manager for social media posts
pub struct HashtagManager {
    standard_hashtags: Vec<String>,
}

impl HashtagManager {
    /// Create a new hashtag manager
    pub fn new() -> Self {
        Self {
            standard_hashtags: vec![
                "#cliptions".to_string(),
                "#ai".to_string(),
                "#CLIP".to_string(),
            ],
        }
    }

    /// Create hashtag manager with custom default hashtags
    pub fn with_defaults(hashtags: Vec<String>) -> Self {
        Self {
            standard_hashtags: hashtags,
        }
    }

    /// Generate hashtags for a round with state information
    pub fn generate_hashtags(
        &self,
        block_num: &str,
        custom_hashtags: Option<Vec<String>>,
    ) -> Vec<String> {
        let mut hashtags = self.standard_hashtags.clone();

        // Add block-specific hashtag
        hashtags.push(format!("#block{}", block_num));

        // Add custom hashtags if provided
        if let Some(custom) = custom_hashtags {
            hashtags.extend(custom);
        }

        hashtags
    }

    /// Generate hashtags for a round with state information
    pub fn generate_hashtags_with_state(
        &self,
        block_num: u64,
        state_name: &str,
        custom_hashtags: Option<Vec<String>>,
    ) -> Vec<String> {
        let mut hashtags = self.standard_hashtags.clone();

        // Add block-specific hashtag
        hashtags.push(format!("#block{}", block_num));

        // Add state-specific hashtag (lowercase, no prefix)
        let state_hashtag = format!("#{}", state_name.to_lowercase());
        hashtags.push(state_hashtag);

        // Add custom hashtags if provided
        if let Some(custom) = custom_hashtags {
            hashtags.extend(custom);
        }

        hashtags
    }

    /// Format hashtags for social media
    pub fn format_hashtags(&self, hashtags: &[String]) -> String {
        hashtags.join(" ")
    }

    /// Extract hashtags from text
    pub fn extract_hashtags(&self, text: &str) -> Vec<String> {
        let hashtag_regex = Regex::new(r"#\w+").unwrap();
        hashtag_regex
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Validate hashtag format
    pub fn validate_hashtag(&self, hashtag: &str) -> bool {
        let hashtag_regex = Regex::new(r"^#\w+$").unwrap();
        hashtag_regex.is_match(hashtag)
    }
}

impl Default for HashtagManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Announcement formatter for different types of round announcements
pub struct AnnouncementFormatter {
    hashtag_manager: HashtagManager,
}

impl AnnouncementFormatter {
    /// Create a new announcement formatter
    pub fn new() -> Self {
        Self {
            hashtag_manager: HashtagManager::new(),
        }
    }

    /// Create announcement formatter with custom hashtag manager
    pub fn with_hashtag_manager(hashtag_manager: HashtagManager) -> Self {
        Self { hashtag_manager }
    }

    /// Create a standard round announcement
    pub fn create_standard_announcement(&self, data: &AnnouncementData) -> String {
        let hashtags = self.hashtag_manager.generate_hashtags_with_state(
            data.block_num,
            &data.state_name,
            None,
        );
        let hashtag_string = self.hashtag_manager.format_hashtags(&hashtags);

        let prize_info = if let Some(prize) = data.prize_pool {
            format!(" Prize pool: {} TAO.", prize)
        } else {
            String::new()
        };

        format!(
            "🎯 Block {} is now live! Target frame reveal at {}.{} Submit your predictions below! {}",
            data.block_num,
            data.target_time,
            prize_info,
            hashtag_string
        )
    }

    /// Create a custom announcement with provided message
    pub fn create_custom_announcement(&self, data: &AnnouncementData) -> String {
        let hashtags = self.hashtag_manager.generate_hashtags_with_state(
            data.block_num,
            &data.state_name,
            Some(data.hashtags.clone()),
        );
        let hashtag_string = self.hashtag_manager.format_hashtags(&hashtags);

        format!("{} {}", data.message, hashtag_string)
    }

    /// Format announcement based on type
    pub fn format_announcement(&self, data: &AnnouncementData, use_custom: bool) -> String {
        if use_custom && !data.message.is_empty() {
            self.create_custom_announcement(data)
        } else {
            self.create_standard_announcement(data)
        }
    }

    /// Create a commitment phase announcement
    pub fn create_commitment_announcement(&self, data: &AnnouncementData) -> String {
        // Generate hashtags with commitment-specific format
        let mut hashtags = vec![
            "#cliptions".to_string(),
            "#ai".to_string(),
            "#CLIP".to_string(),
            format!("#block{}", data.block_num),
            format!("#{}", data.state_name.to_lowercase()),
        ];

        // Add any custom hashtags
        hashtags.extend(data.hashtags.clone());

        let hashtag_string = self.hashtag_manager.format_hashtags(&hashtags);

        let instructions = format!(
            "BLOCK {} - Commitment Phase\n\
            livestream: {}\n\n\
            How To Play:\n\
            1. Generate commitment hash\n\
            2. Reply BEFORE: {}\n\n\
            Reply format ->\nCommit: [hash]\nWallet: [address]",
            data.block_num,
            data.livestream_url.as_deref().unwrap_or(""),
            data.target_time
        );

        format!("{}\n\n{}", hashtag_string, instructions)
    }

    /// Create a reveals phase announcement
    pub fn create_reveals_announcement(&self, data: &AnnouncementData) -> String {
        // Generate hashtags with reveals-specific format
        let mut hashtags = vec![
            "#cliptions".to_string(),
            "#ai".to_string(),
            "#CLIP".to_string(),
            format!("#block{}", data.block_num),
            format!("#{}", data.state_name.to_lowercase()),
        ];

        // Add any custom hashtags
        hashtags.extend(data.hashtags.clone());

        let hashtag_string = self.hashtag_manager.format_hashtags(&hashtags);

        let instructions = format!(
            "BLOCK {} - REVEAL PHASE - Target frame below \n\n\
            Reply to THIS tweet with the unencrypted text of your #block{} commitment before the deadline\n\n\
            Deadline: {}\n\n\
            Use this format:\n\
            Guess: [your-guess]\n\
            Salt: [your-salt]\n\n\
            ",
            data.block_num,
            data.block_num,
            data.target_time
        );

        format!("{}\n\n{}", hashtag_string, instructions)
    }
}

impl Default for AnnouncementFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock social media task for testing
pub struct MockSocialTask {
    name: String,
    should_succeed: bool,
}

impl MockSocialTask {
    /// Create a new mock task that succeeds
    pub fn new(name: String) -> Self {
        Self {
            name,
            should_succeed: true,
        }
    }

    /// Create a mock task that fails
    pub fn new_failing(name: String) -> Self {
        Self {
            name,
            should_succeed: false,
        }
    }
}

impl SocialTask for MockSocialTask {
    fn execute(&self, context: &TaskContext) -> Result<String> {
        if self.should_succeed {
            Ok(format!(
                "Mock task '{}' executed successfully with context: {:?}",
                self.name, context
            ))
        } else {
            Err(CliptionsError::ValidationError(format!(
                "Mock task '{}' failed",
                self.name
            )))
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn validate_parameters(&self, _params: &HashMap<String, String>) -> Result<()> {
        Ok(())
    }
}

/// Social media workflow manager
pub struct SocialWorkflow {
    tasks: Vec<Box<dyn SocialTask>>,
    url_parser: UrlParser,
    announcement_formatter: AnnouncementFormatter,
}

impl SocialWorkflow {
    /// Create a new social workflow
    pub fn new() -> Result<Self> {
        Ok(Self {
            tasks: Vec::new(),
            url_parser: UrlParser::new()?,
            announcement_formatter: AnnouncementFormatter::new(),
        })
    }

    /// Add a task to the workflow
    pub fn add_task(&mut self, task: Box<dyn SocialTask>) {
        self.tasks.push(task);
    }

    /// Execute all tasks in the workflow
    pub fn execute_workflow(&self, contexts: &[TaskContext]) -> Result<Vec<String>> {
        let mut results = Vec::new();

        for (task, context) in self.tasks.iter().zip(contexts.iter()) {
            // Validate parameters first
            task.validate_parameters(&context.parameters)?;

            // Execute the task
            let result = task.execute(context)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Get URL parser
    pub fn get_url_parser(&self) -> &UrlParser {
        &self.url_parser
    }

    /// Get announcement formatter
    pub fn get_announcement_formatter(&self) -> &AnnouncementFormatter {
        &self.announcement_formatter
    }
}

/// Simple tweet cache for reducing Twitter API calls
/// Stores the last validator tweet and only queries Twitter when needed
///
/// Note: `cached_at` is always the local system time when the tweet was cached, not the tweet's original creation time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweetCache {
    pub tweet_id: String,
    pub tweet_text: String,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub validator_username: String,
}

impl TweetCache {
    /// Create a new tweet cache entry
    pub fn new(tweet_id: String, tweet_text: String, validator_username: String) -> Self {
        Self {
            tweet_id,
            tweet_text,
            cached_at: chrono::Utc::now(),
            validator_username,
        }
    }

    /// Check if the cached tweet is still fresh (less than 15 minutes old)
    pub fn is_fresh(&self) -> bool {
        let now = chrono::Utc::now();
        let cache_age = now - self.cached_at;
        cache_age < chrono::Duration::minutes(15)
    }

    /// Check if the tweet contains state hashtags that indicate it's a round state tweet
    pub fn has_state_hashtags(&self) -> bool {
        let hashtag_manager = HashtagManager::new();
        let hashtags = hashtag_manager.extract_hashtags(&self.tweet_text);

        // Look for state hashtags: #cliptions, #block{number}, state-specific hashtags
        let has_cliptions = hashtags.iter().any(|h| h.to_lowercase() == "#cliptions");
        let has_block = hashtags
            .iter()
            .any(|h| h.to_lowercase().starts_with("#block"));
        let has_state = hashtags.iter().any(|h| {
            let h_lower = h.to_lowercase();
            h_lower == "#commitmentsopen"
                || h_lower == "#commitmentsclosed"
                || h_lower == "#revealsopen"
                || h_lower == "#revealsclosed"
                || h_lower == "#payouts"
                || h_lower == "#finished"
        });

        has_cliptions && has_block && has_state
    }
}

pub struct TweetCacheManager {
    cache_file: String,
}

impl TweetCacheManager {
    /// Create a new cache manager with the specified cache file
    pub fn new(cache_file: String) -> Self {
        Self { cache_file }
    }

    /// Create a cache manager with default cache file location
    pub fn default() -> Self {
        Self::new("data/validator_tweet_cache.json".to_string())
    }

    /// Load cached tweet from file
    pub fn load_cache(&self) -> Result<Option<TweetCache>> {
        if !std::path::Path::new(&self.cache_file).exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&self.cache_file).map_err(|e| {
            CliptionsError::ValidationError(format!("Failed to read cache file: {}", e))
        })?;

        if content.trim().is_empty() {
            return Ok(None);
        }

        let cache: TweetCache = serde_json::from_str(&content).map_err(|e| {
            CliptionsError::ValidationError(format!("Failed to parse cache file: {}", e))
        })?;

        Ok(Some(cache))
    }

    /// Save tweet to cache file
    pub fn save_cache(&self, cache: &TweetCache) -> Result<()> {
        // Ensure the directory exists
        if let Some(parent) = std::path::Path::new(&self.cache_file).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CliptionsError::ValidationError(format!("Failed to create cache directory: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(cache).map_err(|e| {
            CliptionsError::ValidationError(format!("Failed to serialize cache: {}", e))
        })?;

        std::fs::write(&self.cache_file, content).map_err(|e| {
            CliptionsError::ValidationError(format!("Failed to write cache file: {}", e))
        })?;

        Ok(())
    }

    /// Clear the cache file
    pub fn clear_cache(&self) -> Result<()> {
        if std::path::Path::new(&self.cache_file).exists() {
            std::fs::remove_file(&self.cache_file).map_err(|e| {
                CliptionsError::ValidationError(format!("Failed to remove cache file: {}", e))
            })?;
        }
        Ok(())
    }

    /// Get cached tweet if it's fresh and has state hashtags
    pub fn get_fresh_state_tweet(&self) -> Result<Option<TweetCache>> {
        if let Some(cache) = self.load_cache()? {
            if cache.is_fresh() && cache.has_state_hashtags() {
                return Ok(Some(cache));
            }
        }
        Ok(None)
    }

    /// Update cache with new tweet data (always uses Utc::now for cached_at)
    pub fn update_cache(
        &self,
        tweet_id: String,
        tweet_text: String,
        validator_username: String,
    ) -> Result<()> {
        let cache = TweetCache::new(tweet_id, tweet_text, validator_username);
        self.save_cache(&cache)
    }
}

impl Default for TweetCacheManager {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tweet_id_from_url() {
        let parser = UrlParser::new().unwrap();

        // Test Twitter URL
        let twitter_url = "https://twitter.com/cliptions_test/status/1234567890";
        let tweet_id = parser.extract_tweet_id(twitter_url).unwrap();
        assert_eq!(tweet_id, "1234567890");

        // Test X URL
        let x_url = "https://x.com/cliptions_test/status/9876543210";
        let tweet_id = parser.extract_tweet_id(x_url).unwrap();
        assert_eq!(tweet_id, "9876543210");

        // Test invalid URL
        let invalid_url = "https://example.com/not-a-tweet";
        assert!(parser.extract_tweet_id(invalid_url).is_err());
    }

    #[test]
    fn test_validate_url() {
        let parser = UrlParser::new().unwrap();

        assert!(parser.validate_url("https://twitter.com/test").is_ok());
        assert!(parser.validate_url("http://example.com").is_ok());
        assert!(parser.validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_extract_domain() {
        let parser = UrlParser::new().unwrap();

        let domain = parser.extract_domain("https://twitter.com/test").unwrap();
        assert_eq!(domain, "twitter.com");

        let domain = parser.extract_domain("https://x.com/test").unwrap();
        assert_eq!(domain, "x.com");
    }

    #[test]
    fn test_generate_hashtags() {
        let hashtag_manager = HashtagManager::new();

        let hashtags = hashtag_manager.generate_hashtags("1", None);
        assert!(hashtags.contains(&"#cliptions".to_string()));
        assert!(hashtags.contains(&"#block1".to_string()));

        let custom_hashtags = vec!["#custom".to_string()];
        let hashtags = hashtag_manager.generate_hashtags("2", Some(custom_hashtags));
        assert!(hashtags.contains(&"#custom".to_string()));
    }

    #[test]
    fn test_custom_hashtags() {
        let custom_defaults = vec!["#customtag".to_string()];
        let hashtag_manager = HashtagManager::with_defaults(custom_defaults);

        let hashtags = hashtag_manager.generate_hashtags("1", None);
        assert!(hashtags.contains(&"#customtag".to_string()));
        assert!(hashtags.contains(&"#block1".to_string()));
    }

    #[test]
    fn test_generate_hashtags_with_state() {
        let hashtag_manager = HashtagManager::new();

        let hashtags = hashtag_manager.generate_hashtags_with_state(5, "CommitmentsOpen", None);
        assert!(hashtags.contains(&"#cliptions".to_string()));
        assert!(hashtags.contains(&"#block5".to_string()));
        assert!(hashtags.contains(&"#commitmentsopen".to_string()));
        assert!(hashtags.contains(&"#CLIP".to_string()));

        let custom_hashtags = vec!["#custom".to_string()];
        let hashtags =
            hashtag_manager.generate_hashtags_with_state(3, "RevealsOpen", Some(custom_hashtags));
        assert!(hashtags.contains(&"#custom".to_string()));
        assert!(hashtags.contains(&"#block3".to_string()));
        assert!(hashtags.contains(&"#revealsopen".to_string()));
    }

    #[test]
    fn test_machine_readable_tweet_format() {
        let formatter = AnnouncementFormatter::new();

        // Test CommitmentsOpen state
        let data = AnnouncementData {
            block_num: 42,
            state_name: "CommitmentsOpen".to_string(),
            target_time: "2024-01-01 12:00:00".to_string(),
            hashtags: vec![],
            message: "Commitments are now open!".to_string(),
            prize_pool: None,
            livestream_url: Some("https://example.com/livestream".to_string()),
        };

        let tweet = formatter.format_announcement(&data, true);

        // Verify machine-readable components
        assert!(
            tweet.contains("#cliptions"),
            "Tweet should contain lowercase #cliptions"
        );
        assert!(
            tweet.contains("#block42"),
            "Tweet should contain block-specific hashtag"
        );
        assert!(
            tweet.contains("#commitmentsopen"),
            "Tweet should contain state hashtag"
        );
        assert!(
            tweet.contains("#CLIP"),
            "Tweet should contain uppercase #CLIP for model reference"
        );
        assert!(tweet.contains("#ai"), "Tweet should contain #ai hashtag");
        assert!(
            !tweet.contains("#predictionmarket"),
            "Tweet should not contain removed hashtag"
        );

        // Test RevealsOpen state with different round
        let data2 = AnnouncementData {
            block_num: 7,
            state_name: "RevealsOpen".to_string(),
            target_time: "2024-01-01 18:00:00".to_string(),
            hashtags: vec![],
            message: "Time to reveal!".to_string(),
            prize_pool: Some(100.0),
            livestream_url: Some("https://example.com/livestream2".to_string()),
        };

        let tweet2 = formatter.format_announcement(&data2, true);
        assert!(
            tweet2.contains("#block7"),
            "Tweet should contain correct block number"
        );
        assert!(
            tweet2.contains("#revealsopen"),
            "Tweet should contain correct state"
        );

        // Verify hashtag format consistency
        assert!(
            !tweet2.contains("#RevealsOpen"),
            "State hashtag should be lowercase"
        );
        assert!(
            !tweet2.contains("#Block7"),
            "Block hashtag should be lowercase"
        );
    }

    #[test]
    fn test_format_hashtags() {
        let hashtag_manager = HashtagManager::new();
        let hashtags = vec!["#tag1".to_string(), "#tag2".to_string()];

        let formatted = hashtag_manager.format_hashtags(&hashtags);
        assert_eq!(formatted, "#tag1 #tag2");
    }

    #[test]
    fn test_extract_hashtags() {
        let hashtag_manager = HashtagManager::new();
        let text = "This is a tweet with #hashtag1 and #hashtag2";

        let hashtags = hashtag_manager.extract_hashtags(text);
        assert_eq!(hashtags, vec!["#hashtag1", "#hashtag2"]);
    }

    #[test]
    fn test_validate_hashtag() {
        let hashtag_manager = HashtagManager::new();

        assert!(hashtag_manager.validate_hashtag("#validhashtag"));
        assert!(hashtag_manager.validate_hashtag("#valid123"));
        assert!(!hashtag_manager.validate_hashtag("invalid"));
        assert!(!hashtag_manager.validate_hashtag("#invalid-tag"));
    }

    #[test]
    fn test_create_standard_round_announcement() {
        let formatter = AnnouncementFormatter::new();
        let data = AnnouncementData {
            block_num: 1,
            state_name: "CommitmentsOpen".to_string(),
            target_time: "2024-01-01 12:00:00".to_string(),
            hashtags: vec![],
            message: "".to_string(),
            prize_pool: Some(100.0),
            livestream_url: None,
        };

        let announcement = formatter.create_standard_announcement(&data);
        assert!(announcement.contains("Block 1 is now live"));
        assert!(announcement.contains("2024-01-01 12:00:00"));
        assert!(announcement.contains("Prize pool: 100 TAO"));
        assert!(announcement.contains("#cliptions"));
        assert!(announcement.contains("#block1"));
        assert!(announcement.contains("#commitmentsopen"));
    }

    #[test]
    fn test_create_custom_round_announcement() {
        let formatter = AnnouncementFormatter::new();
        let data = AnnouncementData {
            block_num: 2,
            state_name: "RevealsOpen".to_string(),
            target_time: "2024-01-01 12:00:00".to_string(),
            hashtags: vec!["#custom".to_string()],
            message: "Custom announcement message".to_string(),
            prize_pool: None,
            livestream_url: None,
        };

        let announcement = formatter.create_custom_announcement(&data);
        assert!(announcement.contains("Custom announcement message"));
        assert!(announcement.contains("#custom"));
        assert!(announcement.contains("#block2"));
        assert!(announcement.contains("#revealsopen"));
    }

    #[test]
    fn test_full_announcement_flow() {
        let formatter = AnnouncementFormatter::new();
        let data = AnnouncementData {
            block_num: 3,
            state_name: "Payouts".to_string(),
            target_time: "2024-01-01 12:00:00".to_string(),
            hashtags: vec![],
            message: "".to_string(),
            prize_pool: Some(50.0),
            livestream_url: None,
        };

        // Test standard announcement
        let standard = formatter.format_announcement(&data, false);
        assert!(standard.contains("Block 3 is now live"));

        // Test custom announcement (should fallback to standard when message is empty)
        let custom = formatter.format_announcement(&data, true);
        assert!(custom.contains("Block 3 is now live"));
    }

    #[test]
    fn test_social_task_execute_success() {
        let task = MockSocialTask::new("test_task".to_string());
        let context = TaskContext {
            task_name: "test_task".to_string(),
            parameters: HashMap::new(),
            timeout_seconds: 30,
        };

        let result = task.execute(&context);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("executed successfully"));
    }

    #[test]
    fn test_social_task_execute_with_kwargs() {
        let task = MockSocialTask::new("test_task".to_string());
        let mut parameters = HashMap::new();
        parameters.insert("param1".to_string(), "value1".to_string());

        let context = TaskContext {
            task_name: "test_task".to_string(),
            parameters,
            timeout_seconds: 30,
        };

        let result = task.execute(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_social_task_failure() {
        let task = MockSocialTask::new_failing("failing_task".to_string());
        let context = TaskContext {
            task_name: "failing_task".to_string(),
            parameters: HashMap::new(),
            timeout_seconds: 30,
        };

        let result = task.execute(&context);
        assert!(result.is_err());
    }

    #[test]
    fn test_social_workflow() {
        let mut workflow = SocialWorkflow::new().unwrap();

        let task1 = Box::new(MockSocialTask::new("task1".to_string()));
        let task2 = Box::new(MockSocialTask::new("task2".to_string()));

        workflow.add_task(task1);
        workflow.add_task(task2);

        let contexts = vec![
            TaskContext {
                task_name: "task1".to_string(),
                parameters: HashMap::new(),
                timeout_seconds: 30,
            },
            TaskContext {
                task_name: "task2".to_string(),
                parameters: HashMap::new(),
                timeout_seconds: 30,
            },
        ];

        let results = workflow.execute_workflow(&contexts).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].contains("task1"));
        assert!(results[1].contains("task2"));
    }

    #[test]
    fn test_announcement_data_validation() {
        let data = AnnouncementData {
            block_num: 1,
            state_name: "Finished".to_string(),
            target_time: "2024-01-01 12:00:00".to_string(),
            hashtags: vec!["#test".to_string()],
            message: "Test message".to_string(),
            prize_pool: Some(100.0),
            livestream_url: None,
        };

        assert_eq!(data.block_num, 1);
        assert_eq!(data.target_time, "2024-01-01 12:00:00");
        assert_eq!(data.hashtags, vec!["#test"]);
        assert_eq!(data.message, "Test message");
        assert_eq!(data.prize_pool, Some(100.0));
    }
}
