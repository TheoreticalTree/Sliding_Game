# Sliding Game
The sliding game is a game about sliding things on blocks.

## Fundamental Idea
There is an NxM square grid. The level is initialized with most squares empty and some squares containing objects.
The player controls k agents which spawn on some of the objects. When on an object a agent can move to an adjacent passable object slide the object they are on a cardinal direction. Sliding an object with an agent on it out of the grid means game over. Sliding into another object usually stops the object siding (see below).
Some objects are marked as goal objects with a number of agents that have to be placed onto. Once the requirements of all goal objects are fulfilled the game is won.

## Objects
### Blocks
 - Blocks can be passable (agents can move onto them) or impassable.
   - All passable objects can be a Goal(g), which means that g agents must be on the object to win
 - Blocks can be slidable (agents can cause them to slide when on them). There are two types of sliding:
    - FastSlide: The object slides arbitrarily far until stopped
    - SlowSlide(l): The object slides l squares or until stopped
 - Blocks have a reaction to an object sliding into them.
    - Stops: The object that slid into stops its movement.
    - Breaks: The object disappears when hit.
    - Bouncy: A bouncy object starts moving when struck. All bouncy objects are Stops.
       - Bouncy: The object FastSlide in the direction of the hitting object
       - Bouncy(l): The object SlowSlide(l) in the direction of the hitting object


#### Impassable Stationary Block
 - Passable: No
 - Slidable: No
 - On Hit: Stops

#### Basic Block
 - Passable: Yes
 - Slidable: FastSlide
 - On Hit: Stops

#### Big Breakable Block
 - Passable: Yes
 - Slidable: FastSlide
 - On Hit: Stops, Breaks 

#### Small Breakable Blocl
 - Passable: Yes
 - Slidable: No
 - On Hit: Breaks

#### Simple Bouncy Block
 - Passable: Yes
 - Slidable: FastSlide
 - On Hit: Stops, Bouncy

### Further Objects
#### Portal
 - When a sliding object would enter the portal it instead leaves the other side of the portal in the appropriate direction

## Implementation:
 - Central Board Struct
    - Keeps track of where agents are
    - Gets user inputs
 - Levels
    - Stored in toml format
    - define fileds:
       - Dimension: two ints
