update-proto:
	git fetch proto main
	git subtree pull --prefix proto/ proto main --squash

set-proto:
	git remote add -f proto git@github.com:asuyou/anzen-proto.git
	git subtree add --prefix proto/ proto main --squash

fmt:
	cargo fmt

