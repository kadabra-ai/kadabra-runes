use kadabra_runes::lsp::client::LspClient;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Represents a test fixture with files and a cursor position.
#[derive(Debug)]
pub struct Fixture {
    /// files in fixture
    pub files: Vec<(PathBuf, String)>,
    /// Position of cursor in fixture
    pub cursor: (PathBuf, u32, u32),
}

/// Parses fixate and covert to file content and paths
/// ## Panics
/// if input is malformed or cursor is not found
pub fn parse_fixture(temp_dir: &TempDir, input: &str) -> Fixture {
    std::fs::create_dir_all(temp_dir.path()).expect("mkdir failed");
    let mut files = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_content = String::new();

    let mut cursor = None;

    for line in input.lines() {
        if let Some(path) = line.strip_prefix("//- ") {
            if let Some(p) = current_path.take() {
                files.push((p, current_content.clone()));
                current_content.clear();
            }
            // Store relative path (trim leading slash)
            let pbuf = PathBuf::from(path.trim_start_matches('/'));
            current_path = Some(pbuf);
        } else {
            let mut l = line.to_string();
            if let Some(idx) = l.find("$0") {
                // Line number is 1-indexed for the API (count + 1)
                let line_no = u32::try_from(current_content.lines().count() + 1)
                    .expect("line count out of range");
                // Column is also 1-indexed for the API (idx + 1)
                let col = u32::try_from(idx + 1).expect("line index out of range");
                cursor = Some((current_path.clone().unwrap(), line_no, col));
                l = l.replace("$0", "");
            }
            current_content.push_str(&l);
            current_content.push('\n');
        }
    }

    if let Some(p) = current_path {
        files.push((p, current_content));
    }

    Fixture {
        files,
        cursor: cursor.expect("missing $0 cursor"),
    }
}

/// Test workspace with optional LSP client
pub struct TestWorkspace {
    /// Temporary folder for the workspace
    pub root: TempDir,
    /// fixture for the workspace
    pub fixture: Fixture,
    /// LSP client (if created)
    pub lsp: Option<LspClient>,
    /// Canonicalized root path (resolves symlinks like /var -> /private/var on macOS)
    canonical_root: PathBuf,
}

impl TestWorkspace {
    /// creates new workspace
    /// ## Panics
    pub fn new(root: TempDir, fixture: &'_ str) -> Self {
        let fixture = parse_fixture(&root, fixture);

        for (path, content) in &fixture.files {
            // path is already relative, just join with root
            let abs = root.path().join(path);
            std::fs::create_dir_all(abs.parent().unwrap()).unwrap();
            std::fs::write(&abs, content).unwrap();
        }

        let canonical_root = root
            .path()
            .canonicalize()
            .expect("Failed to canonicalize root");

        Self {
            root,
            fixture,
            lsp: None,
            canonical_root,
        }
    }

    /// Creates a new builder for constructing a test workspace
    pub fn builder() -> TestWorkspaceBuilder {
        TestWorkspaceBuilder::new()
    }

    /// Returns the canonicalized root path
    pub fn canonical_root(&self) -> &PathBuf {
        &self.canonical_root
    }

    /// Converts a relative path to an absolute path
    pub fn apath(&self, path: &str) -> PathBuf {
        self.canonical_root.join(path)
    }

    /// Returns a reference to the LSP client
    /// ## Panics
    /// Panics if LSP client was not created
    pub fn lsp(&self) -> &LspClient {
        self.lsp
            .as_ref()
            .expect("LSP client not initialized. Use builder().with_lsp() to create LSP client")
    }
}

/// Builder for creating test workspaces with LSP client and optional file opening
pub struct TestWorkspaceBuilder {
    fixture: Option<String>,
    open_files: bool,
}

impl TestWorkspaceBuilder {
    /// Creates a new builder
    pub fn new() -> Self {
        Self {
            fixture: None,
            open_files: false,
        }
    }

    /// Sets the fixture content
    #[must_use]
    pub fn fixture(mut self, fixture: &str) -> Self {
        self.fixture = Some(fixture.to_string());
        self
    }

    /// Enables automatic opening of all files in the workspace via `did_open`
    #[must_use]
    pub fn open_all_files(mut self) -> Self {
        self.open_files = true;
        self
    }

    /// Builds the test workspace with LSP client
    /// ## Panics
    /// Panics if fixture is not set
    pub async fn build(self) -> TestWorkspace {
        let fixture_str = self.fixture.expect("Fixture must be set using .fixture()");

        // Create the workspace with files
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mut workspace = TestWorkspace::new(temp_dir, &fixture_str);

        // Create LSP client
        let lsp = spawn_lsp_for_workspace(&workspace).await;

        // Optionally open all files
        if self.open_files {
            open_workspace_files(&lsp, &workspace).await;
        }

        workspace.lsp = Some(lsp);

        workspace
    }
}

impl Default for TestWorkspaceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to spawn LSP client for a workspace
async fn spawn_lsp_for_workspace(workspace: &TestWorkspace) -> LspClient {
    use super::lsp_harness::spawn_lsp;
    spawn_lsp(workspace.canonical_root()).await
}

/// Helper to open all files in the workspace
async fn open_workspace_files(lsp: &LspClient, workspace: &TestWorkspace) {
    // Open all files from the fixture
    for (relative_path, _) in &workspace.fixture.files {
        let abs_path = workspace.canonical_root.join(relative_path);
        let _ = lsp.did_open(&abs_path).await;
    }

    // Wait a bit for rust-analyzer to process the files
    tokio::time::sleep(Duration::from_millis(1000)).await;
}
