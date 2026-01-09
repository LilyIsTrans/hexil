#include "init_vulkan.h"
#include "hexil.h"
#include <SDL3/SDL_assert.h>
#include <SDL3/SDL_log.h>
#include <SDL3/SDL_vulkan.h>
#include <stdint.h>
#include <stdlib.h>
#include <volk.h>

void hexil_create_global_vulkan_instance(struct HexilGlobalState *appstate) {
  SDL_assert_always(volkInitialize() == VK_SUCCESS);

  // We're incompatible with Vulkan 1.0; any Vulkan greater or equal to 1.1 will
  // have this function, and it's how we actually check the version, so we fail
  // if it's not loaded.
  SDL_assert_always(vkEnumerateInstanceVersion != NULL);

  uint32_t vulkan_instance_version;
  SDL_assert_always(vkEnumerateInstanceVersion(&vulkan_instance_version) == VK_SUCCESS);
  SDL_assert_always(vulkan_instance_version >= VK_API_VERSION_1_4);

  uint32_t SDL_instance_extension_count;
  const char *const *SDL_instance_extensions = SDL_Vulkan_GetInstanceExtensions(&SDL_instance_extension_count);

  const char **const instance_extensions = malloc(sizeof(char *) * (SDL_instance_extension_count + HEXIL_VULKAN_INSTANCE_EXTENSION_COUNT));

  uint32_t instance_extension_count = HEXIL_VULKAN_INSTANCE_EXTENSION_COUNT;

  memcpy(instance_extensions, HEXIL_VULKAN_INSTANCE_EXTENSIONS, sizeof(HEXIL_VULKAN_INSTANCE_EXTENSIONS));

  for (uint_fast32_t i = 0; i < SDL_instance_extension_count; ++i) {
    bool is_already_enabled = false;
    for (uint_fast32_t j = 0; j < HEXIL_VULKAN_INSTANCE_EXTENSION_COUNT; ++j) {
      if (strcmp(SDL_instance_extensions[i], HEXIL_VULKAN_INSTANCE_EXTENSIONS[j]) == 0) {
        is_already_enabled = true;
        break;
      }
    }
    if (is_already_enabled) {
      continue;
    }
    instance_extensions[instance_extension_count++] = SDL_instance_extensions[i];
  }

  if (SDL_GetLogPriority(SDL_LOG_CATEGORY_GPU) >= SDL_LOG_PRIORITY_INFO) {
    SDL_LogInfo(SDL_LOG_CATEGORY_GPU, "SDL Reported the following %" PRIu32 " Vulkan instance extensions are required:\n", SDL_instance_extension_count);
    for (uint32_t i = 0; i < SDL_instance_extension_count; ++i) {
      SDL_LogInfo(SDL_LOG_CATEGORY_GPU, "\t%s\n", SDL_instance_extensions[i]);
    }
  }

  VkApplicationInfo app_info = {
      .sType = VK_STRUCTURE_TYPE_APPLICATION_INFO,
      .pNext = NULL,
      .pApplicationName = "Hexil",
      .applicationVersion = 0,
      .pEngineName = NULL,
      .engineVersion = 0,
      .apiVersion = VK_API_VERSION_1_4,
  };

  VkInstanceCreateInfo instance_create_info = {.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
                                               .pNext = NULL, // TODO (Maybe): Implement support for non-vulkan
                                                              // platforms by attaching a translation-layer driver in a
                                                              // VkDirectDriverLoadingListLUNARG here
                                               .flags = 0,
                                               .pApplicationInfo = NULL,
                                               .enabledLayerCount = 0,
                                               .ppEnabledLayerNames = NULL,
                                               .enabledExtensionCount = SDL_instance_extension_count,
                                               .ppEnabledExtensionNames = SDL_instance_extensions};

  SDL_assert_always(vkCreateInstance(&instance_create_info, NULL, &appstate->vulkan_state.instance) == VK_SUCCESS);
  volkLoadInstanceOnly(appstate->vulkan_state.instance);
}

void swap_physical_devices(uint_fast32_t idx_a, uint_fast32_t idx_b, uint32_t physical_device_count, VkPhysicalDevice physical_devices[physical_device_count],
                           VkPhysicalDeviceProperties2 physical_device_properties[physical_device_count], uint32_t physical_device_queue_family_counts[physical_device_count],
                           uint32_t most_queue_families_per_device,
                           VkQueueFamilyProperties2 physical_device_queue_family_properties[most_queue_families_per_device][physical_device_count]) {
  VkPhysicalDevice temp_dev;
  VkPhysicalDeviceProperties2 temp_prop;
  uint32_t temp_q_fam_count;
  VkQueueFamilyProperties2 temp_q_fams[most_queue_families_per_device];

  memcpy(&temp_dev, &physical_devices[idx_a], sizeof(VkPhysicalDevice));
  memcpy(&temp_prop, &physical_device_properties[idx_a], sizeof(VkPhysicalDeviceProperties2));
  memcpy(&temp_q_fam_count, &physical_device_queue_family_counts[idx_a], sizeof(uint32_t));
  memcpy(&temp_q_fams, &physical_device_queue_family_properties[idx_a], sizeof(VkQueueFamilyProperties2[most_queue_families_per_device]));

  memcpy(&physical_devices[idx_a], &physical_devices[idx_b], sizeof(VkPhysicalDevice));
  memcpy(&physical_device_properties[idx_a], &physical_device_properties[idx_b], sizeof(VkPhysicalDeviceProperties2));
  memcpy(&physical_device_queue_family_counts[idx_a], &physical_device_queue_family_counts[idx_b], sizeof(uint32_t));
  memcpy(&physical_device_queue_family_properties[idx_a], &physical_device_queue_family_properties[idx_b], sizeof(VkQueueFamilyProperties2[most_queue_families_per_device]));

  memcpy(&physical_devices[idx_b], &temp_dev, sizeof(VkPhysicalDevice));
  memcpy(&physical_device_properties[idx_b], &temp_prop, sizeof(VkPhysicalDeviceProperties2));
  memcpy(&physical_device_queue_family_counts[idx_b], &temp_q_fam_count, sizeof(uint32_t));
  memcpy(&physical_device_queue_family_properties[idx_b], &temp_q_fams, sizeof(VkQueueFamilyProperties2[most_queue_families_per_device]));
}

VkPhysicalDevice hexil_select_vulkan_physical_device(struct HexilGlobalState *appstate) {
  uint32_t physical_device_count;
  SDL_assert_paranoid(vkEnumeratePhysicalDevices(appstate->vulkan_state.instance, &physical_device_count, NULL) == VK_SUCCESS);
  VkPhysicalDevice physical_devices[physical_device_count];
  VkPhysicalDeviceProperties2 physical_device_properties[physical_device_count];
  uint32_t physical_device_queue_family_counts[physical_device_count];

  // If this fails, that probably means that a GPU got hotplugged at the precice
  // moment since the last call. That would be *very* cool. Also astronomically
  // unlikely, but checking for it is basically free, so might as well have it
  // let you know if it does!
  SDL_assert_release(vkEnumeratePhysicalDevices(appstate->vulkan_state.instance, &physical_device_count, physical_devices) == VK_SUCCESS);

  uint32_t most_queue_families_per_device;
  for (uint_fast32_t i = 0; i < physical_device_count; ++i) {
    uint32_t queue_family_count;
    vkGetPhysicalDeviceQueueFamilyProperties2(physical_devices[i], &queue_family_count, NULL);

    most_queue_families_per_device = most_queue_families_per_device > queue_family_count ? most_queue_families_per_device : queue_family_count;

    physical_device_queue_family_counts[i] = queue_family_count;
  }

  VkQueueFamilyProperties2 physical_device_queue_family_properties[physical_device_count][most_queue_families_per_device];
  VkQueueFlagBits physical_device_overal_queue_flag_bits[physical_device_count] = {};

  for (uint_fast32_t i = 0; i < physical_device_count; ++i) {
    vkGetPhysicalDeviceQueueFamilyProperties2(physical_devices[i], &most_queue_families_per_device, &physical_device_queue_family_properties[i][0]);
    vkGetPhysicalDeviceProperties2(physical_devices[i], &physical_device_properties[i]);
    for (uint_fast32_t j = 0; j < physical_device_queue_family_counts[i]; ++j) {
      physical_device_overal_queue_flag_bits[i] |= physical_device_queue_family_properties[i][j].queueFamilyProperties.queueFlags;
    }
  }

  uint_fast32_t best_so_far = -1;
  uint_fast32_t total_queues_of_best_so_far;
  for (uint_fast32_t i = 0; i < physical_device_count; ++i) {
    if ((physical_device_overal_queue_flag_bits[i] & (VK_QUEUE_GRAPHICS_BIT | VK_QUEUE_TRANSFER_BIT)) != (VK_QUEUE_GRAPHICS_BIT | VK_QUEUE_TRANSFER_BIT)) {
      continue; // Incompatible
    }
    if (physical_device_properties[i].properties.apiVersion < VK_API_VERSION_1_4) {
      continue; // Incompatible
    }

    bool at_least_one_queue_can_present = false;
    for (uint_fast32_t j = 0; j < physical_device_queue_family_counts[i]; ++j) {
      if (SDL_Vulkan_GetPresentationSupport(appstate->vulkan_state.instance, physical_devices[i], j)) {
        at_least_one_queue_can_present = true;
        break;
      }
    }
    if (!at_least_one_queue_can_present) {
      continue;
    }

    uint_fast32_t total_queues = 0;
    for (uint_fast32_t j = 0; j < physical_device_queue_family_counts[i]; ++j) {
      if ((physical_device_queue_family_properties[i][j].queueFamilyProperties.queueFlags & (VK_QUEUE_GRAPHICS_BIT | VK_QUEUE_TRANSFER_BIT)) != 0)
        total_queues += physical_device_queue_family_properties[i][j].queueFamilyProperties.queueCount;
    }

    if (best_so_far == (uint_fast32_t)(-1)) {
      best_so_far = i;
      total_queues_of_best_so_far = total_queues;
      continue;
    }

    // Heuristics go after this point! For now we'll just use whoever has the
    // MOST QUEUES

    if (total_queues_of_best_so_far < total_queues) {
      total_queues_of_best_so_far = total_queues;
      best_so_far = i;
    }
  }

  return physical_devices[best_so_far];
}

void hexil_init_vulkan_device(struct HexilGlobalState *appstate) {
  VkPhysicalDevice physical_device = hexil_select_vulkan_physical_device(appstate);

  uint32_t queue_family_count;
  vkGetPhysicalDeviceQueueFamilyProperties2(physical_device, &queue_family_count, NULL);

  VkQueueFamilyProperties2 queue_family_properties[queue_family_count];
  vkGetPhysicalDeviceQueueFamilyProperties2(physical_device, &queue_family_count, queue_family_properties);

  uint32_t graphics_queue_family = -1;
  uint32_t transfer_queue_family = -1;
  uint32_t queue_count;
  {
    uint32_t tentative_graphics_queue_family = -1;
    bool there_was_only_one_queue = true;
    for (uint_fast32_t i = 0; i < queue_family_count; ++i) {
      if ((queue_family_properties[i].queueFamilyProperties.queueFlags & (VK_QUEUE_GRAPHICS_BIT)) && graphics_queue_family == (uint32_t)(-1)) {
        graphics_queue_family = i;
      }
      if ((queue_family_properties[i].queueFamilyProperties.queueFlags & (VK_QUEUE_TRANSFER_BIT)) && transfer_queue_family == (uint32_t)(-1)) {
        transfer_queue_family = i;
      }
      if (graphics_queue_family == transfer_queue_family && graphics_queue_family != (uint32_t)(-1) && graphics_queue_family != i) {
        if (queue_family_properties[graphics_queue_family].queueFamilyProperties.queueCount == 1) {
          if ((queue_family_properties[i].queueFamilyProperties.queueFlags & (VK_QUEUE_TRANSFER_BIT))) {
            transfer_queue_family = i;
            there_was_only_one_queue = false;
          } else {
            tentative_graphics_queue_family = i;
          }
        } else {
          there_was_only_one_queue = false;
        }
      }
    }
    if (there_was_only_one_queue && tentative_graphics_queue_family != (uint32_t)(-1) && tentative_graphics_queue_family != graphics_queue_family) {
      graphics_queue_family = tentative_graphics_queue_family;
      there_was_only_one_queue = false;
    }
    queue_count = there_was_only_one_queue ? 1 : 2;
  }

  VkDeviceQueueCreateInfo queue_create_infos[queue_count == 1 || (queue_count == 2 && graphics_queue_family == transfer_queue_family) ? 1 : 2];
  float queue_priorities[2] = {1.0f, 1.0f}; // It's perfectly fine to point to an array of size 2 when an
                                            // array of size 1 is expected.
  queue_create_infos[0].sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
  queue_create_infos[0].pNext = NULL;
  queue_create_infos[0].flags = 0;
  queue_create_infos[0].queueFamilyIndex = graphics_queue_family;
  queue_create_infos[0].queueCount = queue_count == 2 && graphics_queue_family == transfer_queue_family ? 2 : 1;
  queue_create_infos[0].pQueuePriorities = queue_priorities;

  if (queue_count == 2 && graphics_queue_family != transfer_queue_family) {
    queue_create_infos[1].sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
    queue_create_infos[1].pNext = NULL;
    queue_create_infos[1].flags = 0;
    queue_create_infos[1].queueFamilyIndex = transfer_queue_family;
    queue_create_infos[1].queueCount = 1;
    queue_create_infos[1].pQueuePriorities = queue_priorities;
  }

  VkDeviceCreateInfo device_create_info = {
      .sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
      .pNext = NULL,
      .flags = 0,
      .queueCreateInfoCount = queue_count,
      .pQueueCreateInfos = queue_create_infos,
      .enabledLayerCount = 0,
      .ppEnabledLayerNames = NULL,
      .enabledExtensionCount = HEXIL_VULKAN_DEVICE_EXTENSION_COUNT,
      .ppEnabledExtensionNames = HEXIL_VULKAN_DEVICE_EXTENSIONS,
      .pEnabledFeatures = NULL,

  };

  if (appstate->vulkan_state.device != NULL) {
    hexil_free_device(appstate);
  }
  SDL_assert_always(vkCreateDevice(physical_device, &device_create_info, NULL, &appstate->vulkan_state.device) == VK_SUCCESS);
  appstate->vulkan_state.physical_device = physical_device;
  SDL_LogInfo(SDL_LOG_CATEGORY_GPU, "Successfully initialized Vulkan GPU device!");
}

void hexil_create_surface(struct HexilGlobalState *appstate, HexilOsWindow *window) {
  
  SDL_assert_always(SDL_Vulkan_CreateSurface(
      window->window, appstate->vulkan_state.instance, NULL, &window->surface));
}

void hexil_build_swapchain(struct HexilGlobalState *appstate, HexilOsWindow *window) {

  VkSurfacePresentModeKHR present_mode = {.sType = VK_STRUCTURE_TYPE_SURFACE_PRESENT_MODE_KHR, .pNext = NULL, .presentMode = VK_PRESENT_MODE_FIFO_KHR};
  VkPhysicalDeviceSurfaceInfo2KHR surface_info = {.sType = VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SURFACE_INFO_2_KHR, .pNext = &present_mode, .surface = window->surface};

  VkSurfacePresentModeCompatibilityKHR present_mode_compat = {
      .sType = VK_STRUCTURE_TYPE_SURFACE_PRESENT_MODE_COMPATIBILITY_KHR,
      .pNext = NULL,

  };
  VkSurfaceCapabilities2KHR surface_caps = {
      .sType = VK_STRUCTURE_TYPE_SURFACE_CAPABILITIES_2_KHR,
      .pNext = &present_mode_compat,
  };

  SDL_assert_release(vkGetPhysicalDeviceSurfaceCapabilities2KHR(appstate->vulkan_state.physical_device, &surface_info, &surface_caps));

  static const VkImageFormatListCreateInfo fmt_list_create_info [[gnu::common]] = {
      .sType = VK_STRUCTURE_TYPE_IMAGE_FORMAT_LIST_CREATE_INFO,
      .pNext = NULL,
      .viewFormatCount = sizeof(HEXIL_SWAPCHAIN_FORMATS) / sizeof(VkFormat),
      .pViewFormats = HEXIL_SWAPCHAIN_FORMATS,

  };

  VkExtent2D window_size;
  SDL_GetWindowSizeInPixels(window->window, &window_size.width, &window_size.height);
  if (window_size.height <= 0 || window_size.width <= 0) {
    window_size = surface_caps.surfaceCapabilities.maxImageExtent;
  }

  if (surface_caps.surfaceCapabilities.supportedCompositeAlpha & VK_COMPOSITE_ALPHA_POST_MULTIPLIED_BIT_KHR) {
    window->alpha_composite_mode = VK_COMPOSITE_ALPHA_POST_MULTIPLIED_BIT_KHR;
  } else if (surface_caps.surfaceCapabilities.supportedCompositeAlpha & VK_COMPOSITE_ALPHA_PRE_MULTIPLIED_BIT_KHR) {
    window->alpha_composite_mode = VK_COMPOSITE_ALPHA_PRE_MULTIPLIED_BIT_KHR;
  } else if (surface_caps.surfaceCapabilities.supportedCompositeAlpha & VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR) {
    window->alpha_composite_mode = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
  } else if (surface_caps.surfaceCapabilities.supportedCompositeAlpha & VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR) {
    window->alpha_composite_mode = VK_COMPOSITE_ALPHA_INHERIT_BIT_KHR;
  }

  VkSwapchainCreateInfoKHR create_info = {.sType = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
                                          .pNext = &fmt_list_create_info,
                                          .flags = VK_SWAPCHAIN_CREATE_PRESENT_ID_2_BIT_KHR | VK_SWAPCHAIN_CREATE_PRESENT_WAIT_2_BIT_KHR |
                                                   VK_SWAPCHAIN_CREATE_DEFERRED_MEMORY_ALLOCATION_BIT_KHR | VK_SWAPCHAIN_CREATE_MUTABLE_FORMAT_BIT_KHR,
                                          .surface = window->surface,
                                          .minImageCount = surface_caps.surfaceCapabilities.minImageCount,
                                          .imageFormat = HEXIL_SWAPCHAIN_FORMATS[0],
                                          .imageColorSpace = VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                                          .imageExtent =
                                              surface_caps.surfaceCapabilities.currentExtent.height != UINT32_MAX ? surface_caps.surfaceCapabilities.currentExtent : window_size,
                                          .imageArrayLayers = 1,
                                          .imageUsage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
                                          .imageSharingMode = VK_SHARING_MODE_EXCLUSIVE,
                                          .queueFamilyIndexCount = 1,
                                          .pQueueFamilyIndices = NULL,
                                          .preTransform = VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
                                          .compositeAlpha = window->alpha_composite_mode,
                                          .presentMode = present_mode.presentMode,
                                          .clipped = VK_TRUE,
                                          .oldSwapchain = window->swapchain};

  VkSwapchainKHR new_swapchain;
  VkResult result = vkCreateSwapchainKHR(appstate->vulkan_state.device, &create_info, NULL, &new_swapchain);
  // TODO: Use result (handle failure)
  if (result != VK_SUCCESS) {
    SDL_LogCritical(SDL_LOG_CATEGORY_GPU, "Oh boy! Swapchain creation failed! The app will probably crash in a second. Vulkan Error code: %x", result);
  }

  vkDestroySwapchainKHR(appstate->vulkan_state.device, window->swapchain, NULL);


  window->swapchain = new_swapchain;
}


void hexil_allocate_window_cmd_pools(struct HexilGlobalState *appstate, HexilOsWindow *window) {
  
}

void hexil_allocate_window_cmd_buffers(struct HexilGlobalState *appstate, HexilOsWindow *window) {
  
}

void hexil_init_global_vulkan_state(struct HexilGlobalState *appstate) {
  hexil_init_vulkan_device(appstate);
  volkLoadDevice(appstate->vulkan_state.device);
}
