APP = krusty
APP_DIST_DIR = target/debug

LIB_EXT =
INSTALL_PATH =

CP =
DEL =

ifeq ($(OS),Windows_NT)
	CP = xcopy
	DEL = rmdir /s /q
	INSTALL_PATH = $(USERPROFILE)/.$(APP)
	LIB_EXT = dll
else
	CP = cp
	DEL = rm -rf
	INSTALL_PATH = $(HOME)/.$(APP)

    UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
        LIB_EXT = so
    endif
    ifeq ($(UNAME_S),Darwin)
        LIB_EXT = dylib
    endif
endif



all: build

build:
	cargo build

install: build
	mkdir -p $(INSTALL_PATH)
	$(CP) $(APP_DIST_DIR)/$(APP) $(INSTALL_PATH)/$(APP)
# $(CP) $(APP_DIST_DIR)/*.$(LIB_EXT) $(INSTALL_PATH)
	$(INSTALL_PATH)/$(APP) --install test_code/mathlib
	$(INSTALL_PATH)/$(APP) --install $(APP_DIST_DIR)/*.$(LIB_EXT)

uninstall:
	$(DEL) $(INSTALL_PATH)

clean:
	cargo clean
