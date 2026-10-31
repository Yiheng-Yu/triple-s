SPHINXOPTS  ?=
SPHINXBUILD ?= sphinx-build
SOURCEDIR   := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
BUILDDIR    ?= $(SOURCEDIR)_build

.DEFAULT_GOAL := help
.PHONY: help

$(lastword $(MAKEFILE_LIST)): ;

help:
	@$(SPHINXBUILD) -M help "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

%:
	@$(SPHINXBUILD) -M $@ "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)
