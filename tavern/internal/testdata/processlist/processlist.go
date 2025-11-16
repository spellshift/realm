package processlist

import (
	"fmt"
	"math/rand"
	"strings"
	"time"

	"realm.pub/tavern/internal/c2/epb"
)

var (
	// A list of realistic and fantasy process names.
	baseProcessNames = []string{
		"svchost.exe", "csrss.exe", "winlogon.exe", "lsass.exe", "explorer.exe",
		"kernel_task", "launchd", "sysmond", "mds_stores", "WindowServer",
		"systemd", "kthreadd", "gnome-shell", "Xorg", "pulseaudio",
		"chrome", "firefox", "slack", "code", "spotify",
		"docker", "containerd", "kubelet", "etcd",
		"sshd", "login", "bash", "zsh",
		"goblins.exe", "dragonfirewall.service", "necromancer.daemon", "spellchecker.sh",
		"elven-scout", "dwarven-miner", "orc-berserker", "lich-phylactery.sys",
		"grimoire-compiler", "potion-brewer", "alchemist-fumes", "mimic-detector",
		"paladin-proxy", "cleric-healer.service", "rogue-backdoor", "bardic-inspiration.so",
		"tarrasque-ram-eater", "beholder-antivirus", "owlbear-handler", "rust-monster.exe",
	}

	paths = []string{"/bin", "/sbin", "/usr/bin", "/usr/sbin", "/usr/local/bin", "/opt/spells/bin"}
	users = []string{"root", "admin", "gandalf", "bilbo", "aurora", "system", "local_user"}
)

// New creates a new mock process list with a random assortment of processes.
func New() *epb.ProcessList {
	rand.Seed(time.Now().UnixNano())

	numProcesses := rand.Intn(41) + 10 // 10-50 processes
	processes := make([]*epb.Process, 0, numProcesses)
	pids := make(map[uint64]bool)

	// Always add an init process
	pids[1] = true
	processes = append(processes, &epb.Process{
		Pid:  1,
		Ppid: 0,
		Name: "init",
		Path: "/sbin/init",
		Cmd:  "/sbin/init",
		Principal: "root",
	})


	for i := 1; i < numProcesses; i++ {
		var pid uint64
		for {
			pid = uint64(rand.Intn(65534) + 2)
			if !pids[pid] {
				pids[pid] = true
				break
			}
		}

		ppid := uint64(1)
		if len(processes) > 1 {
			ppid = processes[rand.Intn(len(processes))].Pid
		}

		name := baseProcessNames[rand.Intn(len(baseProcessNames))]
		// Add some variation to exceed 1000+ possibilities
		if strings.HasSuffix(name, ".service") {
			name = fmt.Sprintf("systemd-%s-%d", strings.TrimSuffix(name, ".service"), rand.Intn(100))
		} else if strings.HasSuffix(name, ".exe") {
			// no change
		} else {
			name = fmt.Sprintf("%s-%d", name, rand.Intn(1000))
		}

		path := fmt.Sprintf("%s/%s", paths[rand.Intn(len(paths))], name)
		user := users[rand.Intn(len(users))]

		processes = append(processes, &epb.Process{
			Pid:  pid,
			Ppid: ppid,
			Name: name,
			Path: path,
			Cmd:  path + " --verbose",
			Principal: user,
		})
	}

	return &epb.ProcessList{
		List: processes,
	}
}
