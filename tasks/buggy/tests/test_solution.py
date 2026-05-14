import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
from solution import add, multiply, is_palindrome, fizzbuzz

def test_add():
    assert add(2, 3) == 5
    assert add(-1, 1) == 0
    assert add(0, 0) == 0

def test_multiply():
    assert multiply(3, 4) == 12
    assert multiply(0, 5) == 0

def test_palindrome():
    assert is_palindrome("racecar") == True
    assert is_palindrome("hello") == False

def test_fizzbuzz():
    result = fizzbuzz(15)
    assert result[0] == "1"
    assert result[2] == "Fizz"
    assert result[4] == "Buzz"
    assert result[14] == "FizzBuzz"