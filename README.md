# Gateau - Simple HTML Texas Hold'em Server

This is a project that aims to allow poker bots and humans to compete together at Texas Hold'em. 
Any client must be able to `POST` and recieve `POST`s from the server according to the specification below.
The server attempts to correctly recreate the rules of Hold'em; any deviations not listed below should be opened as an issue.

## Interpretation of received game moves
The game will always interpret a legal move from what you `POST`. If you send an illegal move it will be reinterpreted according to the following rules.

### Check
An inappropriate check is a fold.
### Bet
A bet of 0 is ambiguous and will be interpreted as a check if possible, otherwise a fold.
A bet that's less than a call is a call.
A raise that's less than the minimum amount is a min-raise.
A bet that's greater than your stack is an all-in.

## Specification of JSON structures expected by each endpoint
### Received by Server
#### `/config`
This is for configuring the game. 
```
{
    "config" : "property", // What game property to change
    "value"  : 0,          // Value to change property to, if applicable
}
```
`config` can currently be any of the following:
* "starting_stack"
* "max_players"
* "start" to start the game early.
#### `/reg`
Player Registration, `POST` to join the game.
```
{
    "name"    : "Display Name",          // The player's display name
    "address" : "http://127.0.0.1:5000", // The player's return address
}
```
#### `/game`
Game moves are submitted here.
```
{
    "secret_id" : "123e4567-e89b-12d3-a456-426655440000", // UUID that the player must use for confirmation.
    "action"    : "Fold",                                 // Bet, Call, Fold, Check, AllIn
    "value"     : 0,                                      // In the case of bet, the amount to bet, otherwise unused
}
```

### Received by Client
#### `/player`
This endpoint is for the game to `POST` game to. 
The following structures may be sent and should be listened for.
The "info" field is shared by all structures and can be used to determine what the structure represents.
##### PlayerPrivateInfo
This is sent when all players have joined. 
It contains both the player's public in-game ID and its secret ID that it uses for move validation.
```
{
    "info"      : "PlayerPrivateInfo",
    "ingame_id" : usize,               // Public ingame player number
    "secret_id" : String,              // A uuid as a string with dashes
}
```

##### GameTableInfo
This is sent when the game starts.
It contains things like the player move order as well as the starting stacks, etc.
```
{
    "info" : "GameTableInfo"                
    "starting_stack" : usize,               // Chip amounts everyone starts with
    "seat_order" : Vec<usize>,              // Clockwise seat order - cyclical
    "button_player" : usize,                // Player who currently posesses the dealer button
    "display_names" : Vec<(usize, String)>, // Map of ingame IDs to a player-specified name
}
```

##### HoleCardInfo 
This is sent at the start of a hand, informing you of what cards you have.
```
{
    "info" : "HoleCardInfo",         
    "hole_cards" : (String, String), // The two cards that you got
    "hand_number" : usize,           // The hand number
}
```

##### MoveInfo 
This is sent when the game has received and interpreted a move from the player with action.

N.B. "Bet" in this context means taking new chips from your stack and putting them in front of you for any reason.
As such it covers betting, raising, and calling.
```
{
    "info" : "MoveInfo"    
    "player_id" : usize,   // Player that made the move
    "move_type" : String,  // "Bet", "Fold", "Check"
    "value" : usize,       // Value of move if applicable
    "hand_number" : usize, // Current hand number
}
```

##### ToMoveInfo 
This is sent when the game is waiting on a player to move.
```
{
    "info" : "ToMoveInfo", 
    "player_id" : usize,   // Player we're waiting on
    "hand_number" : usize, // Current hand number
}
```

##### StreetInfo 
This is sent when a new street is dealt.
```
{
    "info" : "StreetInfo"
    "street" : String,                    // "PreFlop", "Flop", "Turn", "River"
    "button_player" : usize,              // Player that currently holds the dealer button
    "board_cards_revealed" : Vec<String>, // List of cards that got revealed, if any
    "hand_number" : usize,                // Current hand number
}
```

##### PayoutInfo 
This is sent when a hand has ended and players are paid.
It contains payout info as well as any player hands that became visible on payout.
```
{
    "info" : "PayoutInfo"
    "reason" : String,                            // E.g. "All others folded", "Showdown"
    "payouts" : Vec<(usize, usize)>,              // Player IDs and payout amounts
    "hole_cards" : Vec<(usize,(String, String))>, // Player IDs and revealed cards, if any
}
```

##### PlayerEliminatedInfo 
This is not currently ever sent.
It will be sent to notify a player is eliminated. Currently just look at player stack size.
```
{
    "info" : "PlayerEliminationInfo"
    "eliminated_player" : usize,
}
```

##### GameOverInfo 
This is sent when only one player has any chips and the game is over.
```
{
    "info" : "GameOverInfo"
    "winning_player" : usize,
}
```

## Known deviations from the rules
### Simplifications
* Currently no blind increases
* Currently no antes

### To-dos
* Winner's hand is revealed when everyone else folds
* Players who don't have to reveal hands during showdown should auto-muck their hands

## General To-dos
* Add timer to moves
* Secure configuration endpoint
* Add a database of game info
* Move to asynch player pushes

# License
See `NOTICE` file