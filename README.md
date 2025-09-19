# Subgrader

**Subgrader** is an automatic grading assistant for coding exercises. It helps professors facilitate the grading process by:

- **Downloading** submissions from a Google Classroom assignment
- **Formatting** all submissions into a consistent, predefined structure
- **Detecting** plagiarism and code similarity between submissions
- **Generating** a detailed report with results

## Quick Start

1.  **Clone the repository**

    ``` bash
    > git clone https://github.com/iampassos/subgrader
    > cd subgrader
    ```

2.  **Set up Google Cloud credentials**

    - Go to the [Google Cloud Console](https://console.cloud.google.com/) and generate your Classroom API credentials.
    - Rename the file to `credentials.json` and place it in the project root (`subgrader/`).

3.  **Install Rust**

    - Download and install Rust using [rustup](https://rustup.rs/).

4.  **Run Subgrader**

    ``` bash
    > cargo run --release
    ```

## Notes

-   Requires a valid Google Classroom API setup to download assignments.
-   Reports will include formatting errors, empty files, and detected plagiarism.
-   Beecrowd's .csv report should be placed in the project root.

