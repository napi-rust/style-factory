.PHONY: patch minor major pre-release

patch:
	npm version patch -m "release version %s"

minor:
	npm version minor -m "release version %s"

major:
	npm version major -m "release version %s"

pre-release:
	npm version prerelease --preid=next -m "pre-release version %s"