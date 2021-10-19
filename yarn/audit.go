package yarn

import (
	"encoding/json"
	"fmt"
)

type Audit struct {
	Advisories map[string]Advisory `json:"advisories,omitempty"`
}

type Advisory struct {
	ModuleName      string `json:"module_name,omitempty"`
	PatchedVersions string `json:"patched_versions,omitempty"`
}

func ParseAudit(output []byte) ([]Advisory, error) {
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
