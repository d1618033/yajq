YAJQ
======


Yet Another Json Query language, in Rust!


Usage
=====


```
$ cat sample.json | yajq 
{
  "people": [
    {
      "email": "adams@company.com",
      "name": "Adam Smith"
    },
    {
      "email": "eves@company.com",
      "name": "Eve Smith"
    }
  ]
}

$ cat sample.json | yajq "people"
[
  {
    "email": "adams@company.com",
    "name": "Adam Smith"
  },
  {
    "email": "eves@company.com",
    "name": "Eve Smith"
  }
]

$ cat sample.json | yajq "people.0"
{
  "email": "adams@company.com",
  "name": "Adam Smith"
}

$ cat sample.json | yajq "people.0.email"
"adams@company.com"

$ cat sample.json | yajq "people.*.email"
[
  "adams@company.com",
  "eves@company.com"
]
```
