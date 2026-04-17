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

#define ASTAL_DISPLAYS_TYPE_FAILED_DISPLAY_UPDATE (astal_displays_failed_display_update_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysFailedDisplayUpdate, astal_displays_failed_display_update, ASTAL_DISPLAYS, FAILED_DISPLAY_UPDATE, GObject)

#define ASTAL_DISPLAYS_TYPE_DISPLAY_UPDATE_RESULT (astal_displays_display_update_result_get_type())
G_DECLARE_FINAL_TYPE(AstalDisplaysDisplayUpdateResult, astal_displays_display_update_result, ASTAL_DISPLAYS, DISPLAY_UPDATE_RESULT, GObject)

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
 * astal_displays_manager_query_async:
 * @self: the manager
 * @cancellable: (nullable): cancellable for the async query
 * @callback: callback invoked when the query completes
 * @user_data: user data for @callback
 *
 */
void astal_displays_manager_query_async(AstalDisplaysManager *self, GCancellable *cancellable, GAsyncReadyCallback callback, gpointer user_data);

/**
 * astal_displays_manager_query_finish:
 * @self: the manager
 * @result: async result from #astal_displays_manager_query_async
 * @n_results: (out): number of returned displays
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type AstalDisplaysDisplay): queried displays
 */
AstalDisplaysDisplay **astal_displays_manager_query_finish(AstalDisplaysManager *self, GAsyncResult *result, gsize *n_results, GError **error);

/**
 * astal_displays_manager_get_async:
 * @self: the manager
 * @ids: (array length=n_ids) (element-type AstalDisplaysDisplayIdentifier): requested identifiers
 * @n_ids: number of identifiers
 * @cancellable: (nullable): cancellable for the async lookup
 * @callback: callback invoked when the lookup completes
 * @user_data: user data for @callback
 *
 */
void astal_displays_manager_get_async(AstalDisplaysManager *self, AstalDisplaysDisplayIdentifier **ids, gsize n_ids, GCancellable *cancellable, GAsyncReadyCallback callback, gpointer user_data);

/**
 * astal_displays_manager_get_finish:
 * @self: the manager
 * @result: async result from #astal_displays_manager_get_async
 * @n_results: (out): number of returned matches
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type AstalDisplaysDisplayMatch): resolved display matches
 */
AstalDisplaysDisplayMatch **astal_displays_manager_get_finish(AstalDisplaysManager *self, GAsyncResult *result, gsize *n_results, GError **error);

/**
 * astal_displays_manager_apply_async:
 * @self: the manager
 * @updates: (array length=n_updates) (element-type AstalDisplaysDisplayUpdate): requested updates
 * @n_updates: number of updates
 * @validate: whether to validate without applying
 * @cancellable: (nullable): cancellable for the async update
 * @callback: callback invoked when the update completes
 * @user_data: user data for @callback
 *
 */
void astal_displays_manager_apply_async(AstalDisplaysManager *self, AstalDisplaysDisplayUpdate **updates, gsize n_updates, gboolean validate, GCancellable *cancellable, GAsyncReadyCallback callback, gpointer user_data);

/**
 * astal_displays_manager_apply_finish:
 * @self: the manager
 * @result: async result from #astal_displays_manager_apply_async
 * @n_results: (out): number of update results
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type AstalDisplaysDisplayUpdateResult): apply results
 */
AstalDisplaysDisplayUpdateResult **astal_displays_manager_apply_finish(AstalDisplaysManager *self, GAsyncResult *result, gsize *n_results, GError **error);

/**
 * astal_displays_manager_update_async:
 * @self: the manager
 * @updates: (array length=n_updates) (element-type AstalDisplaysDisplayUpdate): requested updates
 * @n_updates: number of updates
 * @cancellable: (nullable): cancellable for the async update
 * @callback: callback invoked when the update completes
 * @user_data: user data for @callback
 *
 */
void astal_displays_manager_update_async(AstalDisplaysManager *self, AstalDisplaysDisplayUpdate **updates, gsize n_updates, GCancellable *cancellable, GAsyncReadyCallback callback, gpointer user_data);

/**
 * astal_displays_manager_update_finish:
 * @self: the manager
 * @result: async result from #astal_displays_manager_update_async
 * @n_results: (out): number of update results
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type AstalDisplaysDisplayUpdateResult): apply results
 */
AstalDisplaysDisplayUpdateResult **astal_displays_manager_update_finish(AstalDisplaysManager *self, GAsyncResult *result, gsize *n_results, GError **error);

/**
 * astal_displays_manager_validate_async:
 * @self: the manager
 * @updates: (array length=n_updates) (element-type AstalDisplaysDisplayUpdate): requested updates
 * @n_updates: number of updates
 * @cancellable: (nullable): cancellable for the async validation
 * @callback: callback invoked when the validation completes
 * @user_data: user data for @callback
 *
 */
void astal_displays_manager_validate_async(AstalDisplaysManager *self, AstalDisplaysDisplayUpdate **updates, gsize n_updates, GCancellable *cancellable, GAsyncReadyCallback callback, gpointer user_data);

/**
 * astal_displays_manager_validate_finish:
 * @self: the manager
 * @result: async result from #astal_displays_manager_validate_async
 * @n_results: (out): number of update results
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type AstalDisplaysDisplayUpdateResult): apply results
 */
AstalDisplaysDisplayUpdateResult **astal_displays_manager_validate_finish(AstalDisplaysManager *self, GAsyncResult *result, gsize *n_results, GError **error);

G_END_DECLS
