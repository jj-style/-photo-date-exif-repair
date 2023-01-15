.PHONY: help
help:
	@echo "make <arg>"
	@echo "Arguments:"
	@echo "  all"
	@echo "  build"
	@echo "  test"
	@echo "  testdata"
	@echo "  clean"

.PHONY: all
all: build

.PHONY: build
build:
	@cargo build

.PHONY: test
test:
	@cargo test

.PHONY: testdata
testdata:
	@exiftool -overwrite_original -AllDates= test_data/no-dates/*.jpg

.PHONY: clean
clean:
	@cargo clean