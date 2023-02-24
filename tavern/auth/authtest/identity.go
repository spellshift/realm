package authtest

// TestIdentity that is designed to be used in tests to control authentication properties.
type TestIdentity struct {
	Authenticated bool
	Activated     bool
	Admin         bool
}

// String representation of a TestIdentity will be reported as "test_identity".
func (t TestIdentity) String() string {
	return "test_identity"
}

// IsAuthenticated returns true if the test has configured it to.
func (t TestIdentity) IsAuthenticated() bool {
	return t.Authenticated
}

// IsActivated returns true if the test has configured it to.
func (t TestIdentity) IsActivated() bool {
	return t.Activated
}

// IsAdmin returns true if the test has configured it to.
func (t TestIdentity) IsAdmin() bool {
	return t.Admin
}

// NewAllPowerfulIdentityForTest returns an authenticated, activated, admin identity for tests.
func NewAllPowerfulIdentityForTest() TestIdentity {
	return TestIdentity{true, true, true}
}
