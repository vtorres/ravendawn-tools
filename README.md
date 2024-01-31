<h1 align="center">
  <picture>
    <img src="./assets/icon.png" alt="Habemus Lux" />
  </picture>
</h1>

<div align="center">

![Rust](https://img.shields.io/badge/Rust-EF4A00.svg?style=flat-square)

</div>

<h3 align="center">
  Revendawn Tools
</h3>

<div align="center">
  <picture>
    <img src="./assets/ravendawn.png" alt="Ravendawn" />
  </picture>
</div>

### 📖 About

Ravendawn Online is a massively multiplayer online role-playing game (MMORPG) created by Tavernlight Games.
We could discuss about this "creation", because they are using the whole source code, networking protocol and everything on top of Tibia. Being more specific, they are using known open source like [this](https://github.com/edubart/otclient) and [this](https://github.com/edubart/otclient) and built on top of it. It's funny because they still get mad when you call this game a Tibia copy, haha. Let's avoid this rabbit hole and start hacking it!

### 📖 How to find the right memory address for the Entity

CT File: [Enjoy!](https://github.com/vtorres/ravendawn-tools-private/blob/main/ravendawn.CT)

There are many ways to find the Entity Address, but the easiest way to find it inside this game is using the player direction [North, East, South and West]. Following this tip we could find the Entity base address in less than one minute. The directions have the following values: North: 0, East: 1, South: 2, West: 3

Having this base information gonna save you a lot of time. Let's start!

Login in your account character, attach the Cheat Engine and place your character looking to the North and scan the memory for the value 0, now we can just alternate the directions and filter it by the diriection values provided above.

It's pretty straightforward and after a couple of times doing this steps and filtering the memory we are capable of finding it in less than one minute. After finding the direction memory, we can just start a pointer scan with deep of one. It's gonna show you basically one result and that makes the things easier. Removing this offset and getting the base address it's what we are looking for,
the Entity itself! After findind the Entity base address and looking into the Memory View, you gonna notice everything that we need is there and not to far from the base address, like: 
Current HP, Current SP, Max HP, Max Mana, Directions, Outfits, Player Name, Coordinates (x, y, z), Current XP, Lights, Player ID, Monster ID that we are attacking and much more.

### 🏴‍☠️ Basic Tools

I won't implement things that totally breaks the game like insta mining, insta fishing and others. I'm here just passing basic informations and you can go further! I love doing cheats, but when it comes to 
open a code to the community, things could go south. If you really want to break the game, you easily do Bots or descrypt the lua files of the game and changing the whole behavior.

Now that we are on the same page, let's implement a few basic tools, like Ravendawn Cave Light, Ravendawn Outfit Changer and Ravendawn EXP Tracker.

Ravendawn Cave Light - It's a pain in the ass having a limited vision inside the caves. Let's get rid of it!

<div align="center">
  <picture>
    <img src="./assets/ravendawn.png" alt="Ravendawn Cave Light" />
  </picture>
</div>

Ravendawn Outfit Changer - The game released a week ago, so we do not have a lot of different outfits for our character, unless... hehe

<div align="center">
  <picture>
    <img src="./assets/ravendawn.png" alt="Ravendawn Outfit Changer" />
  </picture>
</div>

Ravendawn EXP Tracker - Sometimes you just want to know more information about your hunting inside the game, the EXP average per hour and how long until reaching the next level. Since the game does not provide us with this info yet.. we need to do it ourselves

<div align="center">
  <picture>
    <img src="./assets/ravendawn.png" alt="Ravendawn EXP Tracker" />
  </picture>
</div>

### 📕 Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)

### 🖥️ Development

```shell
cargo update stable
cargo run
```
or

```shell
cargo update stable
cargo build --release
```

### 🏴‍☠️ Credits

- [Vitor Torres](https://github.com/vtorres/)