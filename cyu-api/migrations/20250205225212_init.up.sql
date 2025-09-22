DROP TABLE IF EXISTS `icstokens`;
CREATE TABLE `icstokens` (
  `id` INTEGER NOT NULL PRIMARY KEY,
  `userid` INTEGER NOT NULL,
  `token` TEXT NOT NULL
);
