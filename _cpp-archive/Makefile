
BUILD_DIR ?= ./build
SRC_DIRS ?= ./src/components

SRCS := $(shell find $(SRC_DIRS) -name *.cpp -or -name *.c -or -name *.s)
OBJS := $(SRCS:%=$(BUILD_DIR)/%.o)
DEPS := $(OBJS:.o=.d)

INC_DIRS := $(shell find $(SRC_DIRS) -type d) \
	./src/third_parties/myhtml/include
INC_FLAGS := $(addprefix -I,$(INC_DIRS))

CPPFLAGS ?= $(INC_FLAGS) -MMD -MP
CXXFLAGS ?= -std=c++11
LDFLAGS ?= 

# assembly
$(BUILD_DIR)/%.s.o: %.s
	$(MKDIR_P) $(dir $@)
	$(AS) $(ASFLAGS) -c $< -o $@

# c++ source
$(BUILD_DIR)/%.cpp.o: %.cpp
	$(MKDIR_P) $(dir $@)
	$(CXX) $(CPPFLAGS) $(CXXFLAGS) -c $< -o $@

# MyHTML library as a sub-project
.myHtml:
	$(MAKE) -C src/third_parties/myhtml MyCORE_BUILD_WITHOUT_THREADS=YES static

.cleanMyhtml:
	$(MAKE) -C src/third_parties/myhtml clean

standalone: $(OBJS) .myHtml
	$(CXX) $(OBJS) src/third_parties/myhtml/lib/libmyhtml_static.a -o $(BUILD_DIR)/speedreader $(LDFLAGS)

.PHONY: clean


clean: .cleanMyhtml
	$(RM) -r $(BUILD_DIR)

-include $(DEPS)

MKDIR_P ?= mkdir -p
