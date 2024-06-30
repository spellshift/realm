package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"io/fs"
	"io/ioutil"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/pkg/browser"
	"gopkg.in/yaml.v2"
	"realm.pub/tavern/cli/auth"
)

type Metadata struct {
	Name         string     `yaml:"name"`
	Description  string     `yaml:"description"`
	Author       string     `yaml:"author"`
	SupportModel string     `yaml:"support_model"`
	Tactic       string     `yaml:"tactic"`
	ParamDefs    []ParamDef `yaml:"paramdefs"`
}

type ParamDef struct {
	Name        string `yaml:"name" json:"name"`
	Type        string `yaml:"type" json:"type"`
	Label       string `yaml:"label" json:"label"`
	Placeholder string `yaml:"placeholder" json:"placeholder"`
}

// Define a struct to represent a file.
type File struct {
	Name    string
	Content []byte
}

// main function to run the application.
func main() {
	// Setup
	basedir := os.Getenv("basedir")
	if basedir == "" {
		basedir = "tomes"
	}
	endpoint := os.Getenv("endpoint")
	graphqlEndpoint := ""
	if endpoint == "" {
		graphqlEndpoint = "http://localhost/graphql"
		endpoint = "http://127.0.0.1"
	} else {
		graphqlEndpoint = endpoint + "/graphql"
	}
	cookie := os.Getenv("cookie")
	if cookie == "" {
		ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
		defer cancel()

		// Configure Browser (uses the default system browser)
		browser := auth.BrowserFunc(browser.OpenURL)

		// Open Browser and Obtain Access Token (via 127.0.0.1 redirect)
		token, err := auth.Authenticate(ctx, browser, endpoint)
		if err != nil {
			panic(err)
		}
		cookie = token.String()
	}

	// Call the function to upload tomes
	if err := UploadTomesGraphQL(basedir, graphqlEndpoint, cookie); err != nil {
		fmt.Println("Error:", err)
	}
}

// UploadTomesGraphQL uploads tomes using GraphQL.
func UploadTomesGraphQL(basedir string, graphqlEndpoint string, cookie string) error {
	dir, _ := os.ReadDir(basedir)
	entries, err := os.ReadDir(basedir)
	if err != nil {
		return fmt.Errorf("failed to read filesystem: %w", err)
	}

	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}

		tomeName := entry.Name()

		// Check if the tome already exists
		if exists, err := checkTomeExists(graphqlEndpoint, tomeName, cookie); err != nil {
			return err
		} else if exists {
			//todo add updating the existing tome
			continue
		}

		// ... inside the loop over entries
		metadata, eldritchContent, tomeFiles, err := processTomeDirectory(dir, basedir+"/"+tomeName)
		if err != nil {
			return err
		}

		// Create the tome using GraphQL with the metadata and eldritchContent
		if err := createTomeGraphQL(graphqlEndpoint, metadata, eldritchContent, tomeFiles, cookie); err != nil {
			return err
		}
		// ...

		// // Create the tome using GraphQL
		// if err := createTomeGraphQL(graphqlEndpoint, metadata, tomeName, tomeFiles); err != nil {
		// 	return err
		// }
	}

	return nil
}

// checkTomeExists checks if a tome already exists using a GraphQL query.
func checkTomeExists(endpoint, tomeName string, cookie string) (bool, error) {
	query := fmt.Sprintf(`query CheckTomeExists {
		tomes(where: {
			name: "%s"
	
		}) {
			id
		}
	}
	`, tomeName)
	response, err := sendGraphQLRequest(endpoint, query, cookie)
	if err != nil {
		return false, err
	}

	// Extract the 'exists' field from the response
	var result struct {
		Data struct {
			Tomes []string `json:"tomes"`
		}
	}
	if err := json.Unmarshal(response, &result); err != nil {
		return false, fmt.Errorf("failed to parse GraphQL response: %w", err)
	}
	if len(result.Data.Tomes) > 0 {
		return true, nil
	}
	return false, nil
}

// processTomeDirectory reads files in a tome directory and prepares them for upload.
func processTomeDirectory(fs []fs.DirEntry, dirName string) (Metadata, string, []File, error) {
	var files []File
	var metadata Metadata
	var eldritchContent string

	err := filepath.WalkDir(dirName, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() {
			return nil
		}

		content, err := ioutil.ReadFile(path)
		if err != nil {
			return fmt.Errorf("failed to read file %q: %w", path, err)
		}

		switch filepath.Base(path) {
		case "metadata.yml":
			if err := yaml.Unmarshal(content, &metadata); err != nil {
				return fmt.Errorf("failed to parse metadata.yml: %w", err)
			}
		case "main.eldritch":
			eldritchContent = string(content)
		default:
			files = append(files, File{Name: filepath.Base(path), Content: content})
		}

		return nil
	})

	if err != nil {
		return Metadata{}, "", nil, fmt.Errorf("failed to process tome directory: %w", err)
	}

	return metadata, eldritchContent, files, nil
}

// createTomeGraphQL sends a GraphQL mutation to create a new tome with its files.
func createTomeGraphQL(endpoint string, metadata Metadata, eldritchContent string, files []File, cookie string) error {
	// Encode eldritchContent in base64 for safe transmission
	escapedEldritchContent := strings.ReplaceAll(eldritchContent, "\\", "\\\\")
	escapedEldritchContent = strings.ReplaceAll(escapedEldritchContent, "\"", "\\\"")
	escapedEldritchContent = strings.ReplaceAll(escapedEldritchContent, "\n", "\\n")

	// Create files first and collect their IDs
	// fileIDs := make([]string, 0)
	// for _, file := range files {
	// 	mutation := fmt.Sprintf(`
	//         mutation {
	//             createFile(name: "%s", content: "%s") {
	//                 id
	//             }
	//         }
	//     `, file.Name, base64.StdEncoding.EncodeToString(file.Content))

	// 	resp, err := sendGraphQLRequest(endpoint, mutation)
	// 	if err != nil {
	// 		return fmt.Errorf("failed to create file %q: %w", file.Name, err)
	// 	}

	// 	var result struct {
	// 		Data struct {
	// 			File struct {
	// 				ID string `json:"id"`
	// 			} `json:"createFile"`
	// 		}
	// 	}
	// 	if err := json.Unmarshal(resp, &result); err != nil {
	// 		return fmt.Errorf("failed to parse create file response: %w", err)
	// 	}
	// 	fileIDs = append(fileIDs, result.Data.File.ID)
	// }

	// Convert paramDefs to a JSON string
	paramDefsJSON, err := json.Marshal(metadata.ParamDefs)
	if err != nil {
		return fmt.Errorf("failed to marshal paramDefs: %w", err)
	}
	paramDefsJSONString := string(paramDefsJSON)

	// Escape double quotes for proper string embedding in GraphQL
	paramDefsJSONString = strings.ReplaceAll(paramDefsJSONString, "\"", "\\\"")

	// Construct the GraphQL mutation for creating a tome
	createTomeMutation := fmt.Sprintf(`
        mutation {
            createTome(input: {
                name: "%s",
                description: "%s",
                author: "%s",
                supportModel: %s,
                tactic: %s,
                paramDefs: "%s",
                eldritch: "%s"

            }) {
                id
            }
        }
    `, metadata.Name, metadata.Description, metadata.Author, metadata.SupportModel, metadata.Tactic, paramDefsJSONString, escapedEldritchContent)
	fmt.Println(createTomeMutation)
	// Send the createTome mutation
	_, err = sendGraphQLRequest(endpoint, createTomeMutation, cookie)
	if err != nil {
		return fmt.Errorf("failed to create tome %q: %w", metadata.Name, err)
	}

	return nil
}

func sendGraphQLRequest(endpoint string, query string, cookie string) ([]byte, error) {
	// Marshal the query into a JSON request body
	requestBody, err := json.Marshal(map[string]string{
		"query": query,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to create request body: %w", err)
	}

	// Create a new HTTP request with the necessary headers
	req, err := http.NewRequest("POST", endpoint, bytes.NewBuffer(requestBody))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	// Set the content type and cookie headers
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Cookie", cookie)

	// Create a new HTTP client and send the request
	client := &http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer resp.Body.Close()

	// Check the response status code
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("unexpected status code: %d", resp.StatusCode)
	}

	// Read and return the response body using io.ReadAll
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	return body, nil
}
