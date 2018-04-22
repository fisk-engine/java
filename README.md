## Lait

A low-pitched, better scripting-language language.

### Syntax

Lait's syntax is aimed towards being a superior scripting language, specifically designed for game development. The idioms are primariy borrowed from Lua and Rust, e.g. the power of tables and the *implement* keyword.

```
player: {
  x: float
  y: float
  
  health: int = 10
}

enemy: {
  x: float
  y: float
  
  damage: int = 1
}

anders := player {
  x: 100
  y: 50
}

günther := enemy {
  x: 10
  y: 100
}



fun move(thing: {x: float, y: float}, x: float, y: float) = {
  thing x += x
  thing y += y
}

move(anders, 10, 10)
move(günther, 100, 100)
```

### Disclaimer

The Lait compiler and virtual machine is developed is completely developed by kids without further education. Please use this.
