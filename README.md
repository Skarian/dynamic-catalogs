Example Cyberflix Catalog:
https://cyberflix.elfhosted.com/c/catalogs=cd492,15846,cf003,eba63,c4e72,071c0,47f38,71418,61f57,60f26,88681,a2ff4%7Clang=en/catalog/Netflix/netflix.popular.movie.json

Example Trakt Catalog:
https://2ecbbd610840-trakt.baby-beamup.club/eyJsaXN0cyI6WyJ0cmFrdF9wb3B1bGFyIiwidHJha3RfdHJlbmRpbmciLCJ0cmFrdF9zZWFyY2giXSwiaWRzIjpbImdpbGFkZzpsYXRlc3QtcmVsZWFzZXM6YWRkZWQsZGVzYyIsImRvbnh5Om1hcnZlbC1jaW5lbWF0aWMtdW5pdmVyc2U6cmFuayxhc2MiXSwiYWNjZXNzX3Rva2VuIjoiYjc4ZTdlOTE5MDVhMjViY2JlMjFhMjdmMDkxYzBhODI0YzYyYmYxMDFiMDBkMzA2YTgyOTY5ZTcwMDgwNDIyMSIsInJlZnJlc2hfdG9rZW4iOiIyZDYyYjgyOWY4ZWM2NjllZDE2YTczYWQ2NGE0NjEzNTQwNDVhMmViMDFkMTkwZjJmYjk1MTMzMTdjMTY0ZTlkIiwiZXhwaXJlcyI6MTczOTE4NjEyNiwicmVtb3ZlUHJlZml4IjpmYWxzZX0=/catalog/trakt/trakt_popular_movies.json

Example Trakt Addon Entry:

```
{
 "id": "tt1431045",
 "type": "movie",
 "name": "Deadpool",
 "poster": "https://images.metahub.space/poster/small/tt1431045/img",
 "background": "https://images.metahub.space/background/medium/tt1431045/img",
 "releaseInfo": "2016",
 "description": "The origin story of former Special Forces operative turned mercenary Wade Wilson, who, after being subjected to a rogue experiment that leaves him with accelerated healing powers, adopts the alter ego Deadpool. Armed with his new abilities and a dark, twisted sense of humor, Deadpool hunts down the man who nearly destroyed his life.",
 "genres": [
  "action",
  "adventure",
  "comedy",
  "superhero"
 ],
 "trailers": [
  {
   "source": "9vN6DHB6bJc",
   "type": "Trailer"
  }
 ],
 "behaviorHints": {
  "defaultVideoId": "tt1431045"
 }
}

```

Example Trakt API response (extended_info ON):

```
{
 "id": 1052227769,
 "listed_at": "2024-07-08T18:23:41.000Z",
 "movie": {
  "available_translations": [
   "ar",
   "az",
   "bg",
   "ca",
   "cs",
   "da",
   "de",
   "el",
   "en",
   "es",
   "fa",
   "fi",
   "fr",
   "he",
   "hr",
   "hu",
   "id",
   "it",
   "ja",
   "ka",
   "ko",
   "lt",
   "nl",
   "pl",
   "pt",
   "ro",
   "ru",
   "sk",
   "sl",
   "sr",
   "sv",
   "th",
   "tr",
   "uk",
   "vi",
   "zh"
  ],
  "certification": "PG-13",
  "comment_count": 104,
  "country": "us",
  "genres": [
   "action",
   "science-fiction",
   "adventure"
  ],
  "homepage": "http://www.20thcenturystudios.com/movies/kingdom-of-the-planet-of-the-apes",
  "ids": {
   "imdb": "tt11389872",
   "slug": "kingdom-of-the-planet-of-the-apes-2024",
   "tmdb": 653346,
   "trakt": 488280
  },
  "language": "en",
  "languages": [
   "en"
  ],
  "overview": "Several generations following Caesar's reign, apes – now the dominant species – live harmoniously while humans have been reduced to living in the shadows. As a new tyrannical ape leader builds his empire, one young ape undertakes a harrowing journey that will cause him to question all he's known about the past and to make choices that will define a future for apes and humans alike.",
  "rating": 7.13339,
  "released": "2024-05-10",
  "runtime": 145,
  "status": "released",
  "tagline": "No one can stop the reign.",
  "title": "Kingdom of the Planet of the Apes",
  "trailer": "https://youtube.com/watch?v=Tg1FesR8X90",
  "updated_at": "2024-11-11T17:28:41.000Z",
  "votes": 9956,
  "year": 2024
 },
 "notes": null,
 "rank": 1,
 "type": "movie"
}
```

# Realistic sorting options

"Trending Now"

X-Sort-By: popularity, X-Sort-How: desc
"New Releases"

X-Sort-By: released, X-Sort-How: desc
"A-Z"

X-Sort-By: title, X-Sort-How: asc
"Short & Sweet"

X-Sort-By: runtime, X-Sort-How: asc
"Top Rated"

X-Sort-By: percentage, X-Sort-How: desc
"Recently Watched"

X-Sort-By: watched, X-Sort-How: desc
"Fan Favorites"

X-Sort-By: votes, X-Sort-How: desc

# Trakt List - Todo List

- Add sorting
- Add Genres
- Add intial metadata
