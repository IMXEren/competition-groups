# WCA: Competition Groups

Demo project to scrape the data on [competitiongroups.com](https://www.competitiongroups.com) site using Rust.

## Setup

Follow the setup below to use in Termux and Termux: Tasker -

1. Install [Termux](https://f-droid.org/en/packages/com.termux/) from f-droid.
2. Install [Termux: Tasker](https://f-droid.org/en/packages/com.termux.tasker/) from f-droid.
3. Grant permission to Tasker to run commands in Termux environment.
4. Open Termux and run these commands: (with description)

    ```bash
    termux-setup-storage                          ## Allowing access to /storage/emulated/0 (or /sdcard)
    yes | pkg up -y                               ## Upgrade existing packages
    yes | pkg install -y tur-repo x11-repo        ## Adding additional repos to extend package list
    yes | pkg install -y chromium                 ## Installing chromium browser with chromedriver

    ## Set `allow-external-apps` to true
    curl -s https://raw.githubusercontent.com/IMXEren/automation/main/scripts/allow_external_apps.sh | bash

    ## Download & setup zip package
    ## TODO

    ## Running the executable
    ## TODO
    ```
