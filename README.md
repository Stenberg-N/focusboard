# FocusBoard

[![License: MIT](https://img.shields.io/badge/License-MIT-purple.svg)](https://opensource.org/licenses/MIT)
[![Latest Release](https://img.shields.io/github/v/release/Stenberg-N/focusboard?label=Latest)](https://github.com/Stenberg-N/focusboard/releases)

Primarily a note taking app, but also implements a calendar and a timer. Still very much lacking in many features. Plan is to add a local, small AI model that is completely optional to make summarizations from texts, e.g. pick important dates from an email.

## Screenshots

### Home page
<img width="1834" height="1059" alt="focusboard7" src="https://github.com/user-attachments/assets/6ef08405-9748-4990-87ed-d22fd53e791d" />

### Notes
<img width="1919" height="1081" alt="focusboard1" src="https://github.com/user-attachments/assets/a0e28022-5dd4-4c85-b09e-462056fade50" />

### Timer
<img width="1701" height="1000" alt="focusboard2" src="https://github.com/user-attachments/assets/3cd51c4c-eebf-490c-9d6b-c295761da3e1" />

### Calendar
<img width="1701" height="999" alt="focusboard3" src="https://github.com/user-attachments/assets/a50219c3-dae3-40cc-9596-4bac7b2c92d9" />

### Calendar horizontal and vertical timelines

<img width="3439" height="1415" alt="focusboard5" src="https://github.com/user-attachments/assets/5742a048-58f1-46e5-9f1b-5ca4488bb702" />
<img width="1834" height="1058" alt="Näyttökuva 2026-03-22 122116" src="https://github.com/user-attachments/assets/93b31edb-5d60-47f1-8908-7ec1128eb118" />

## Installation

### App installer

1. Go to [releases](https://github.com/Stenberg-N/focusboard/releases)
2. Download the setup executable file. **Note!** **NOT** the one ending with .sig!
3. Install and launch.

### Repository
You need to install Rust. You can do that [here](https://rustup.rs/)<br>
Another thing you need is Microsoft Visual Studio C++ Build Tools, which can be found [here](https://visualstudio.microsoft.com/visual-cpp-build-tools/)<br>
And also you need Node.js, found [here](https://nodejs.org/en)

1. Clone the repo
```bash
git clone https://github.com/Stenberg-N/focusboard.git
cd focusboard
```
2. Install dependencies
```bash
npm install
```
3. Run it
```bash
npm run tauri dev
```

## Other
This app saves the database and logs in:<br> C:\Users\Your_username\AppData\Local\com.stenberg.focusboard\ <br><br>
If you need to find the file where the app saves e.g. your current/latest tab you were on, you can find it at: <br> C:\Users\Your_username\AppData\Roaming\com.stenberg.focusboard\

## Acknowledgements
The icons used for the app's UI uses icons from Uicons by <a href="https://www.flaticon.com/uicons">Flaticon</a>

## License
This project is licensed under the MIT license. See the LICENSE file for details.
