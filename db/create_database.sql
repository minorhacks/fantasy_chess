CREATE DATABASE fantasy_chess;

CREATE USER 'fantasy-chess-rw'@'localhost' IDENTIFIED BY 'faceGuy';

GRANT ALL PRIVILEGES ON fantasy_chess.* TO 'fantasy-chess-rw'@'localhost';

UPDATE mysql.user SET host='%' WHERE User = 'fantasy-chess-rw';

FLUSH PRIVILEGES;

USE fantasy_chess;

