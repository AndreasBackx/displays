#pragma once

#include <gio/gio.h>
#include <glib-object.h>

G_BEGIN_DECLS

typedef enum {
    ASTAL_DISPLAYS_ORIENTATION_LANDSCAPE = 0,
    ASTAL_DISPLAYS_ORIENTATION_PORTRAIT = 90,
    ASTAL_DISPLAYS_ORIENTATION_LANDSCAPE_FLIPPED = 180,
    ASTAL_DISPLAYS_ORIENTATION_PORTRAIT_FLIPPED = 270,
} AstalDisplaysOrientation;

#define ASTAL_DISPLAYS_TYPE_ORIENTATION (astal_displays_orientation_get_type())
GType astal_displays_orientation_get_type(void) G_GNUC_CONST;

#define ASTAL_DISPLAYS_TYPE_MANAGER (astal_displays_manager_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysManager, astal_displays_manager, ASTAL_DISPLAYS, MANAGER, GObject)

#define ASTAL_DISPLAYS_TYPE_DISPLAY_IDENTIFIER (astal_displays_display_identifier_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysDisplayIdentifier, astal_displays_display_identifier, ASTAL_DISPLAYS, DISPLAY_IDENTIFIER, GObject)

#define ASTAL_DISPLAYS_TYPE_POINT (astal_displays_point_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysPoint, astal_displays_point, ASTAL_DISPLAYS, POINT, GObject)

#define ASTAL_DISPLAYS_TYPE_LOGICAL_DISPLAY (astal_displays_logical_display_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysLogicalDisplay, astal_displays_logical_display, ASTAL_DISPLAYS, LOGICAL_DISPLAY, GObject)

#define ASTAL_DISPLAYS_TYPE_LOGICAL_DISPLAY_UPDATE_CONTENT (astal_displays_logical_display_update_content_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysLogicalDisplayUpdateContent, astal_displays_logical_display_update_content, ASTAL_DISPLAYS, LOGICAL_DISPLAY_UPDATE_CONTENT, GObject)

#define ASTAL_DISPLAYS_TYPE_PHYSICAL_DISPLAY (astal_displays_physical_display_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysPhysicalDisplay, astal_displays_physical_display, ASTAL_DISPLAYS, PHYSICAL_DISPLAY, GObject)

#define ASTAL_DISPLAYS_TYPE_PHYSICAL_DISPLAY_UPDATE_CONTENT (astal_displays_physical_display_update_content_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysPhysicalDisplayUpdateContent, astal_displays_physical_display_update_content, ASTAL_DISPLAYS, PHYSICAL_DISPLAY_UPDATE_CONTENT, GObject)

#define ASTAL_DISPLAYS_TYPE_DISPLAY (astal_displays_display_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysDisplay, astal_displays_display, ASTAL_DISPLAYS, DISPLAY, GObject)

#define ASTAL_DISPLAYS_TYPE_DISPLAY_UPDATE (astal_displays_display_update_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysDisplayUpdate, astal_displays_display_update, ASTAL_DISPLAYS, DISPLAY_UPDATE, GObject)

#define ASTAL_DISPLAYS_TYPE_DISPLAY_MATCH (astal_displays_display_match_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysDisplayMatch, astal_displays_display_match, ASTAL_DISPLAYS, DISPLAY_MATCH, GObject)

/**
 * astal_displays_get_default:
 *
 * Returns: (transfer full): the default manager instance
 */
AstalDisplaysManager *astal_displays_get_default(void);

/**
 * astal_displays_manager_get_default:
 *
 * Returns: (transfer full): the default manager instance
 */
AstalDisplaysManager *astal_displays_manager_get_default(void);

/**
 * astal_displays_manager_query:
 * @self: the manager
 * @n_results: (out): number of returned displays
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type AstalDisplaysDisplay): queried displays
 */
AstalDisplaysDisplay **astal_displays_manager_query(AstalDisplaysManager *self, gsize *n_results, GError **error);

/**
 * astal_displays_manager_get:
 * @self: the manager
 * @ids: (array length=n_ids) (element-type AstalDisplaysDisplayIdentifier): requested identifiers
 * @n_ids: number of identifiers
 * @n_results: (out): number of returned matches
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type AstalDisplaysDisplayMatch): resolved display matches
 */
AstalDisplaysDisplayMatch **astal_displays_manager_get(AstalDisplaysManager *self, AstalDisplaysDisplayIdentifier **ids, gsize n_ids, gsize *n_results, GError **error);

/**
 * astal_displays_manager_apply:
 * @self: the manager
 * @updates: (array length=n_updates) (element-type AstalDisplaysDisplayUpdate): requested updates
 * @n_updates: number of updates
 * @validate: whether to validate without applying
 * @n_results: (out): number of remaining updates
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type AstalDisplaysDisplayUpdate): unresolved updates
 */
AstalDisplaysDisplayUpdate **astal_displays_manager_apply(AstalDisplaysManager *self, AstalDisplaysDisplayUpdate **updates, gsize n_updates, gboolean validate, gsize *n_results, GError **error);

/**
 * astal_displays_manager_update:
 * @self: the manager
 * @updates: (array length=n_updates) (element-type AstalDisplaysDisplayUpdate): requested updates
 * @n_updates: number of updates
 * @n_results: (out): number of remaining updates
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type AstalDisplaysDisplayUpdate): unresolved updates
 */
AstalDisplaysDisplayUpdate **astal_displays_manager_update(AstalDisplaysManager *self, AstalDisplaysDisplayUpdate **updates, gsize n_updates, gsize *n_results, GError **error);

/**
 * astal_displays_manager_validate:
 * @self: the manager
 * @updates: (array length=n_updates) (element-type AstalDisplaysDisplayUpdate): requested updates
 * @n_updates: number of updates
 * @n_results: (out): number of remaining updates
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type AstalDisplaysDisplayUpdate): unresolved updates
 */
AstalDisplaysDisplayUpdate **astal_displays_manager_validate(AstalDisplaysManager *self, AstalDisplaysDisplayUpdate **updates, gsize n_updates, gsize *n_results, GError **error);

G_END_DECLS
