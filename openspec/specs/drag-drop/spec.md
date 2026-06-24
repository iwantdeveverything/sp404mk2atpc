# Spec: Drag and Drop

## ADDED Requirements

### Requirement: Native Tauri Drag/Drop
> The application MUST support dragging and dropping audio files directly from the operating system onto the pad interface using Tauri's native events.

#### Scenario: Single valid audio file drop
- **Given** the user has a valid audio file selected in their file explorer
- **When** the user drags and drops the file onto an empty pad
- **Then** the file is loaded and assigned to that pad

### Requirement: Sequential Multi-file Loading
> Dropping multiple audio files onto a single pad MUST sequentially load the files starting from the target pad and continuing to subsequent pads.

#### Scenario: Dropping multiple files onto a pad
- **Given** the user has selected 3 valid audio files
- **When** the user drags and drops them onto Pad 1
- **Then** the first file is loaded to Pad 1
- **And** the second file is loaded to Pad 2
- **And** the third file is loaded to Pad 3

#### Scenario: Dropping multiple files exceeding bank limit
- **Given** the user drops multiple files starting at Pad 15
- **When** the sequence exceeds the final pad of the bank (Pad 16)
- **Then** the loading stops at Pad 16 and any remaining files are discarded

### Requirement: Visual Hover Feedback
> Pads MUST visually indicate when a dragged file is hovering over them to provide drop-target feedback.

#### Scenario: Hovering a dragged file over a pad
- **Given** the user is dragging a file within the application window
- **When** the cursor enters the hit area of a pad
- **Then** the pad visually highlights or glows
- **And** when the cursor leaves the hit area, the visual highlight is removed

### Requirement: LCD Error Messages for Invalid Formats
> Dropping unsupported file types (e.g., non-audio files like PDFs) MUST display a transient error on the LCD.

#### Scenario: Dropping an unsupported file
- **Given** the user drags an unsupported file (e.g., document.pdf)
- **When** the user drops the file onto a pad
- **Then** the file is rejected and not loaded into the pad
- **And** the LCD screen displays an error message such as "Invalid Format"
- **And** the error message clears automatically after a short duration

### Requirement: Multi-File Drag and Drop Feedback

**Requirement:** The application MUST provide clear, detailed feedback via the LCD display and pad animations when multiple files are dropped onto the pads.

**Scenarios:**

*   **Given** a user drags and drops multiple audio files (e.g., 3 files) onto a pad (e.g., Pad 1)
    **When** the files are successfully processed and loaded into consecutive pads (e.g., Pads 1, 2, 3)
    **Then** the LCD display MUST show a detailed summary message indicating the number of files and the range of pads (e.g., "LOADED 3 IN 1-3")
    **And** the LCD message MUST remain visible for 2 seconds before reverting.

*   **Given** a multi-file drop has just completed successfully
    **When** the files are mapped to their respective pads
    **Then** the UI MUST trigger a cascading lighting animation sequentially across the pads that received the new files
    **And** the animation MUST visually reinforce where the files landed, lighting up each target pad one after the other.
