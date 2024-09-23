# PSU
[![Chat on discord](https://img.shields.io/discord/1183477399208345771?color=blue8&label=Discord&logo=discord&style=for-the-badge)](https://discord.gg/qtjrKnCk8y)

---

## What is this
Psu - Password storage utility. And this is my first sql project,so far, he can only
insert passwords and output them to the console, but I plan to improve him.

---

## Quickstart
```
psu add <SERVICE> <LOGIN> <PASSWORD> // add your password to the database
psu print -a // prints all passwords and from the id
psu prind <ID> // if you want to output one specific password
```
The functionality is being finalized and some things may be changed

---

## Assembling
You must have a rust installed then clone and run the build  :
```
  git clone https://github.com/IwantHAPPINESS/psu.git
  cargo build --release
```
and get the file from the target folder and run.
