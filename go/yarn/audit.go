package yarn

import (
	"encoding/json"
	"fmt"
	"gnarl/semver"
	"io"
	"strings"
)

type Audit struct {
	Advisories map[string]Advisory `json:"advisories,omitempty"`
}

type Advisory struct {
	ModuleName      string `json:"module_name,omitempty"`
	PatchedVersions string `json:"patched_versions,omitempty"`
}

func ParseAudit(output []byte, version *semver.Version) ([]Advisory, error) {
	switch version.Major {
	case 2:
	case 3:
		return ParseAuditYarn2(output)
	case 4:
		return ParseAuditYarn4(output)
	default:
		return nil, fmt.Errorf("unsupported yarn version: %v", version)
	}

	// WTF
	return nil, fmt.Errorf("unreachable code for yarn version: %v", version)
}

func ParseAuditYarn2(output []byte) ([]Advisory, error) {
	audit := Audit{}
	err := json.Unmarshal(output, &audit)
	if err != nil {
		return nil, fmt.Errorf("cannot deserialize audit json: %v", err)
	}

	a := make([]Advisory, 0, len(audit.Advisories))

	for _, value := range audit.Advisories {
		a = append(a, value)
	}

	return a, nil
}

type Yarn4Advisory struct {
	ModuleName string                `json:"value"`
	Children   Yarn4AdvisoryChildren `json:"children"`
}

type Yarn4AdvisoryChildren struct {
	Id                 interface{} `json:"ID"`
	VulnerableVersions string      `json:"Vulnerable Versions"`
}

func (yarn4Advisory Yarn4Advisory) ToAdvisory() Advisory {
	return Advisory{
		ModuleName:      yarn4Advisory.ModuleName,
		PatchedVersions: semver.MustParseRequest(yarn4Advisory.Children.VulnerableVersions).Patches().String(),
	}
}

func ParseAuditYarn4(output []byte) ([]Advisory, error) {
	var advisories []Advisory
	dec := json.NewDecoder(strings.NewReader(string(output)))
	for {
		var issue Yarn4Advisory
		err := dec.Decode(&issue)
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, fmt.Errorf("cannot deserialize audit json: %v", err)
		}

		switch v := issue.Children.Id.(type) {
		case string:
			if strings.Contains(v, " (deprecation)") {
				continue
			}
		}

		advisories = append(advisories, issue.ToAdvisory())
	}
	return advisories, nil
}
