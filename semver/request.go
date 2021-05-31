package semver

import (
	"fmt"
	"log"
	"regexp"
	"strconv"
	"strings"
)

func MustParseRequest(request string) *Request {
	result, err := ParseRequest(request)
	if err != nil {
		log.Fatalf("failed to parse request %s: %v", request, err)
	}

	return result
}

func ParseRequest(request string) (*Request, error) {
	var terms []RequestTerm
	for _, part := range strings.Split(request, "||") {
		part = strings.TrimSpace(part)
		term, err := parseRequestTerm(part)
		if err != nil {
			return nil, fmt.Errorf("invalid term %s: %v", part, err)
		}

		terms = append(terms, term)
	}

	return &Request{terms: terms}, nil
}

func parseRequestTerm(term string) (RequestTerm, error) {
	source := regexp.MustCompile(`([<>=~^])\s+`).ReplaceAllString(term, `$1`)
	source = regexp.MustCompile(`\s+`).ReplaceAllString(source, " ")

	var factors []RequestFactor
	for _, part := range strings.Split(source, " ") {
		var constraint Constraint
		switch {
		case part == "*":
			factors = append(factors, RequestFactor{Constraint: Any})
			continue
		case strings.HasPrefix(part, "^"):
			constraint, part = MatchMajor, part[1:]
		case strings.HasPrefix(part, "~"):
			constraint, part = MatchMinor, part[1:]
		case strings.HasPrefix(part, ">="):
			constraint, part = AtLeast, part[2:]
		case strings.HasPrefix(part, "<"):
			constraint, part = Less, part[1:]
		case strings.HasPrefix(part, ">"):
			constraint, part = Greater, part[1:]
		default:
			if _, err := strconv.ParseInt(source, 10, 0); err == nil {
				constraint = MatchMajor
			} else {
				constraint = Exact
			}
		}

		if strings.Contains(part, ".x") {
			constraint, part = MatchMajor, strings.ReplaceAll(part, ".x", ".0")
		}

		version, err := ParseVersion(part)
		if err != nil {
			return nil, err
		}

		if version.Major == 0 && constraint == MatchMajor {
			constraint = MatchMinor
		}

		if version.Major == 0 && version.Minor == 0 && constraint == MatchMinor {
			constraint = Exact
		}

		factors = append(factors, RequestFactor{Constraint: constraint, Version: *version})
	}

	return factors, nil
}

func (r *Request) Matches(version *Version) bool {
terms:
	for _, term := range r.terms {
		for _, factor := range term {
			if !factor.Matches(version) {
				continue terms
			}
		}

		return true
	}

	return false
}

func (r *RequestFactor) Matches(version *Version) bool {
	matchPre := func(strict bool) bool {
		return r.Pre == version.Pre || !strict && version.Pre == ""
	}

	switch r.Constraint {
	case Exact:
		return r.Major == version.Major && r.Minor == version.Minor && r.Patch == version.Patch && matchPre(true)
	case MatchMinor:
		return r.Major == version.Major && r.Minor == version.Minor && r.Patch <= version.Patch && matchPre(false)
	case MatchMajor:
		return r.Major == version.Major &&
			(r.Minor < version.Minor || r.Minor == version.Minor && r.Patch <= version.Patch) && matchPre(false)
	case AtLeast:
		return (r.Major < version.Major || r.Major == version.Major &&
			(r.Minor < version.Minor || r.Minor == version.Minor && r.Patch <= version.Patch)) && matchPre(false)
	case Greater:
		return (r.Major < version.Major || r.Major == version.Major &&
			(r.Minor < version.Minor || r.Minor == version.Minor && r.Patch < version.Patch)) && matchPre(false)
	case Less:
		return (r.Major > version.Major || r.Major == version.Major &&
			(r.Minor > version.Minor || r.Minor == version.Minor && r.Patch > version.Patch)) && matchPre(false)
	case Any:
		return true
	default:
		return false
	}
}

func (r *Request) Overlaps(other *Request) (bool, *Version) {
	fromVersions := other.fromVersions()
	for _, version := range fromVersions {
		if r.Matches(&version) {
			return true, nil
		}
	}

	if len(fromVersions) == 0 {
		return false, nil
	}

	return false, &fromVersions[0]
}

func (r *Request) fromVersions() []Version {
	var versions []Version

	for _, term := range r.terms {
		for _, factor := range term {
			if factor.Constraint == Greater {
				successor := Version{Major: factor.Major, Minor: factor.Minor, Patch: factor.Patch + 1}
				versions = append(versions, successor)
			} else if factor.Constraint != Less && factor.Constraint != Any {
				versions = append(versions, factor.Version)
			}
		}
	}

	return versions
}

type Request struct {
	terms []RequestTerm
}

type RequestTerm []RequestFactor

type RequestFactor struct {
	Version
	Constraint Constraint
}

type Constraint int

const (
	Exact Constraint = iota
	MatchMinor
	MatchMajor
	AtLeast
	Less
	Greater
	Any
)
