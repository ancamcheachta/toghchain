# Toghcháin Éireann

[![Build Status](https://travis-ci.org/ancamcheachta/toghchain.svg?branch=master)](https://travis-ci.org/ancamcheachta/toghchain)

Irish election data

## Purpose
The primary goal of this project is to **open-source all election information
available on the island of Ireland**.

Secondary supporting aims include, but aren't necessarily limited to:

* **Maintaining all election data in a commonly used format (JSON)**
* **Providing tools for common tasks (eg. scrapers, database scripts, etc.)**
* **Publishing experimental services that feature the data in insightful ways**

## Status
### Data collection
Progress in this area is as follows:

* [x] Dáil Éireann
* [ ] Northern Assembly
* [ ] Westminster (north of Ireland)

### Data integrity
Little spot-checking has been performed on data gathered to date, so 
contributions in this area are welcome.  The data is incomplete in certain
areas.

For example, many results from early in the Free State contain little or no 
count information, often reflecting only who was elected.

**Note**: the model for election results is currently undocumented, even though
we are a portion of the way through data collection.  The model specification
will eventually be published in `/src`.

### Tools
As it stands, the only tool released is the database helper 
[`mongoloid`](src/mongoloid). `mongoloid` is responsible for validating the 
Travis build of `toghchain`, and is also the fastest and easiest way to get a
working copy of an Irish election database up and running on your own instance 
of MongoDB.

There are some scrapers in existence that were used to gather the data for the
initial commit, but these are being refactored prior to release.

As work on the Assembly and Westminster hasn't kicked off yet, ideas for
scrapers in these areas are welcome.  Anyone with such an idea is encouraged to
drop it in [Issues](https://github.com/ancamcheachta/toghchain/issues) for the 
moment.

### Experimental services
This is the most underdeveloped portion of the project.  A Rest API is planned,
but not started.