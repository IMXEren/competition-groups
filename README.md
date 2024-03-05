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
   mkdir -p ~/competition-groups && cd ~/competition-groups && arch=arm64 && curl -L -o "cgroups-${arch}.zip" "https://github.com/IMXEren/competition-groups/releases/download/Assets/cgroups-${arch}.zip" && unzip -o -d . cgroups-${arch}.zip && chmod 744 ./compgroups

   ## Running the executable
   cd ~/competition-groups
   WCA_USER="user" \  ## user id or email
   WCA_PASS="pass" \  ## password
   ./compgroups

   ## Output
   Login Successful
   Time elapsed: 43.679098574s
   Done...
   ## Anything other than them concludes to the error.
   ## On success, it should create a competitions.json which has the scraped data.
   ```

## Credits

**[HunterXProgrammer@MdTestV5](https://github.com/HunterXProgrammer/Tasker-MdtestV5)**
