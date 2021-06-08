package semver_test

import (
	"gnarl/semver"
	"testing"
)

func TestParseRequestTermMinusRange(t *testing.T) {
	request, err := semver.ParseRequest("0.28.0 - 0.30.0")
	if err != nil {
		t.Errorf("Parse failed: %v", err)
	}

	t.Logf("Parsed %v", request)

	for _, inside := range []string{"0.28.0", "0.28.3", "0.29.5", "0.30.0"} {
		if !request.Matches(semver.MustParseVersion(inside)) {
			t.Errorf("Must match %s", inside)
		}
	}

	for _, outside := range []string{"0.25.99", "0.30.1"} {
		if request.Matches(semver.MustParseVersion(outside)) {
			t.Errorf("Must not match %s", outside)
		}
	}
}
