#pragma once

#include <gio/gio.h>
#include <glib-object.h>

G_BEGIN_DECLS

typedef enum {
    DISPLAYS_ASTAL_ORIENTATION_LANDSCAPE = 0,
    DISPLAYS_ASTAL_ORIENTATION_PORTRAIT = 90,
    DISPLAYS_ASTAL_ORIENTATION_LANDSCAPE_FLIPPED = 180,
    DISPLAYS_ASTAL_ORIENTATION_PORTRAIT_FLIPPED = 270,
} DisplaysAstalOrientation;

#define DISPLAYS_ASTAL_TYPE_ORIENTATION (displays_astal_orientation_get_type())
GType displays_astal_orientation_get_type(void) G_GNUC_CONST;

#define DISPLAYS_ASTAL_TYPE_MANAGER (displays_astal_manager_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalManager, displays_astal_manager, DISPLAYS_ASTAL, MANAGER, GObject)

#define DISPLAYS_ASTAL_TYPE_DISPLAY_IDENTIFIER (displays_astal_display_identifier_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalDisplayIdentifier, displays_astal_display_identifier, DISPLAYS_ASTAL, DISPLAY_IDENTIFIER, GObject)

#define DISPLAYS_ASTAL_TYPE_POINT (displays_astal_point_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalPoint, displays_astal_point, DISPLAYS_ASTAL, POINT, GObject)

#define DISPLAYS_ASTAL_TYPE_LOGICAL_DISPLAY (displays_astal_logical_display_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalLogicalDisplay, displays_astal_logical_display, DISPLAYS_ASTAL, LOGICAL_DISPLAY, GObject)

#define DISPLAYS_ASTAL_TYPE_LOGICAL_DISPLAY_UPDATE_CONTENT (displays_astal_logical_display_update_content_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalLogicalDisplayUpdateContent, displays_astal_logical_display_update_content, DISPLAYS_ASTAL, LOGICAL_DISPLAY_UPDATE_CONTENT, GObject)

#define DISPLAYS_ASTAL_TYPE_PHYSICAL_DISPLAY (displays_astal_physical_display_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalPhysicalDisplay, displays_astal_physical_display, DISPLAYS_ASTAL, PHYSICAL_DISPLAY, GObject)

#define DISPLAYS_ASTAL_TYPE_PHYSICAL_DISPLAY_UPDATE_CONTENT (displays_astal_physical_display_update_content_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalPhysicalDisplayUpdateContent, displays_astal_physical_display_update_content, DISPLAYS_ASTAL, PHYSICAL_DISPLAY_UPDATE_CONTENT, GObject)

#define DISPLAYS_ASTAL_TYPE_DISPLAY (displays_astal_display_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalDisplay, displays_astal_display, DISPLAYS_ASTAL, DISPLAY, GObject)

#define DISPLAYS_ASTAL_TYPE_DISPLAY_UPDATE (displays_astal_display_update_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalDisplayUpdate, displays_astal_display_update, DISPLAYS_ASTAL, DISPLAY_UPDATE, GObject)

#define DISPLAYS_ASTAL_TYPE_FAILED_DISPLAY_UPDATE (displays_astal_failed_display_update_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalFailedDisplayUpdate, displays_astal_failed_display_update, DISPLAYS_ASTAL, FAILED_DISPLAY_UPDATE, GObject)

#define DISPLAYS_ASTAL_TYPE_DISPLAY_UPDATE_RESULT (displays_astal_display_update_result_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalDisplayUpdateResult, displays_astal_display_update_result, DISPLAYS_ASTAL, DISPLAY_UPDATE_RESULT, GObject)

#define DISPLAYS_ASTAL_TYPE_DISPLAY_MATCH (displays_astal_display_match_get_type())
G_DECLARE_FINAL_TYPE(DisplaysAstalDisplayMatch, displays_astal_display_match, DISPLAYS_ASTAL, DISPLAY_MATCH, GObject)

/**
 * displays_astal_get_default:
 *
 * Returns: (transfer full): the default manager instance
 */
DisplaysAstalManager *displays_astal_get_default(void);

/**
 * displays_astal_manager_get_default:
 *
 * Returns: (transfer full): the default manager instance
 */
DisplaysAstalManager *displays_astal_manager_get_default(void);

/**
 * displays_astal_manager_query_async:
 * @self: the manager
 * @cancellable: (nullable): cancellable for the async query
 * @callback: callback invoked when the query completes
 * @user_data: user data for @callback
 *
 */
void displays_astal_manager_query_async(DisplaysAstalManager *self, GCancellable *cancellable, GAsyncReadyCallback callback, gpointer user_data);

/**
 * displays_astal_manager_query_finish:
 * @self: the manager
 * @result: async result from #displays_astal_manager_query_async
 * @n_results: (out): number of returned displays
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type DisplaysAstalDisplay): queried displays
 */
DisplaysAstalDisplay **displays_astal_manager_query_finish(DisplaysAstalManager *self, GAsyncResult *result, gsize *n_results, GError **error);

/**
 * displays_astal_manager_get_async:
 * @self: the manager
 * @ids: (array length=n_ids) (element-type DisplaysAstalDisplayIdentifier): requested identifiers
 * @n_ids: number of identifiers
 * @cancellable: (nullable): cancellable for the async lookup
 * @callback: callback invoked when the lookup completes
 * @user_data: user data for @callback
 *
 */
void displays_astal_manager_get_async(DisplaysAstalManager *self, DisplaysAstalDisplayIdentifier **ids, gsize n_ids, GCancellable *cancellable, GAsyncReadyCallback callback, gpointer user_data);

/**
 * displays_astal_manager_get_finish:
 * @self: the manager
 * @result: async result from #displays_astal_manager_get_async
 * @n_results: (out): number of returned matches
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type DisplaysAstalDisplayMatch): resolved display matches
 */
DisplaysAstalDisplayMatch **displays_astal_manager_get_finish(DisplaysAstalManager *self, GAsyncResult *result, gsize *n_results, GError **error);

/**
 * displays_astal_manager_apply_async:
 * @self: the manager
 * @updates: (array length=n_updates) (element-type DisplaysAstalDisplayUpdate): requested updates
 * @n_updates: number of updates
 * @validate: whether to validate without applying
 * @cancellable: (nullable): cancellable for the async update
 * @callback: callback invoked when the update completes
 * @user_data: user data for @callback
 *
 */
void displays_astal_manager_apply_async(DisplaysAstalManager *self, DisplaysAstalDisplayUpdate **updates, gsize n_updates, gboolean validate, GCancellable *cancellable, GAsyncReadyCallback callback, gpointer user_data);

/**
 * displays_astal_manager_apply_finish:
 * @self: the manager
 * @result: async result from #displays_astal_manager_apply_async
 * @n_results: (out): number of update results
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type DisplaysAstalDisplayUpdateResult): apply results
 */
DisplaysAstalDisplayUpdateResult **displays_astal_manager_apply_finish(DisplaysAstalManager *self, GAsyncResult *result, gsize *n_results, GError **error);

/**
 * displays_astal_manager_update_async:
 * @self: the manager
 * @updates: (array length=n_updates) (element-type DisplaysAstalDisplayUpdate): requested updates
 * @n_updates: number of updates
 * @cancellable: (nullable): cancellable for the async update
 * @callback: callback invoked when the update completes
 * @user_data: user data for @callback
 *
 */
void displays_astal_manager_update_async(DisplaysAstalManager *self, DisplaysAstalDisplayUpdate **updates, gsize n_updates, GCancellable *cancellable, GAsyncReadyCallback callback, gpointer user_data);

/**
 * displays_astal_manager_update_finish:
 * @self: the manager
 * @result: async result from #displays_astal_manager_update_async
 * @n_results: (out): number of update results
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type DisplaysAstalDisplayUpdateResult): apply results
 */
DisplaysAstalDisplayUpdateResult **displays_astal_manager_update_finish(DisplaysAstalManager *self, GAsyncResult *result, gsize *n_results, GError **error);

/**
 * displays_astal_manager_validate_async:
 * @self: the manager
 * @updates: (array length=n_updates) (element-type DisplaysAstalDisplayUpdate): requested updates
 * @n_updates: number of updates
 * @cancellable: (nullable): cancellable for the async validation
 * @callback: callback invoked when the validation completes
 * @user_data: user data for @callback
 *
 */
void displays_astal_manager_validate_async(DisplaysAstalManager *self, DisplaysAstalDisplayUpdate **updates, gsize n_updates, GCancellable *cancellable, GAsyncReadyCallback callback, gpointer user_data);

/**
 * displays_astal_manager_validate_finish:
 * @self: the manager
 * @result: async result from #displays_astal_manager_validate_async
 * @n_results: (out): number of update results
 * @error: return location for a #GError
 *
 * Returns: (transfer full) (array length=n_results) (element-type DisplaysAstalDisplayUpdateResult): apply results
 */
DisplaysAstalDisplayUpdateResult **displays_astal_manager_validate_finish(DisplaysAstalManager *self, GAsyncResult *result, gsize *n_results, GError **error);

G_END_DECLS
