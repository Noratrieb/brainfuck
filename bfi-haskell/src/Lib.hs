module Lib
    ( main
    ) where

import System.Environment
import Data.List
import System.IO

main :: IO ()
main = do
   args <- getArgs
   program <- readFile $ head args
   interpret program

data Memory = Memory [Int] Int [Int]

interpret :: String -> IO ()
interpret [] = IO()
interpret [x:xs] = do
  eval x
  interpret xs


eval :: String -> Memory -> IO Memory
eval s (Memory sx x xs)
| x == "+" = Memory sx (x + 1) xs
| x == "-" = Memory sx (x - 1) xs
otherwise = undefined
eval _ = undefined