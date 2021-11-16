package yarn

import (
	json "encoding/json"
	"fmt"
	"io/ioutil"
	"os"
)

type Package struct {
	Resolutions map[string]string `json:"resolutions,omitempty"`
}

func packageJson(directory string) string {
	return fmt.Sprintf("%s/package.json", directory)
}

func ReadPackage(directory string) (*Package, error) {
	reader, err := os.Open(packageJson(directory))
	if err != nil {
		return nil, fmt.Errorf("cannot open package.json: %v", err)
	}

	defer reader.Close()

	packageJson, err := ioutil.ReadAll(reader)
	if err != nil {
		return nil, fmt.Errorf("cannot read package.json: %v", err)
	}

	packages := Package{}
	err = json.Unmarshal(packageJson, &packages)
	if err != nil {
		return nil, fmt.Errorf("cannot deserialize package.json: %v", err)
	}

	return &packages, nil
}
