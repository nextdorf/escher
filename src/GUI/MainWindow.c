#include "MainWindow.h"

static void print_hello (GtkWidget *widget, gpointer data){
  g_print ("Hello World\n");
}

static void quit_cb (GtkWindow *window){
  gtk_window_close (window);
}


void activateMainWindow (GtkApplication *app, gpointer user_data){
  /* Construct a GtkBuilder instance and load our UI description */
  GtkBuilder *builder = gtk_builder_new ();
  //gtk_builder_add_from_file (builder, "example.ui", NULL);

  //gtk_builder_add_from_resource (builder, "/gui/GUI/GResource/MainWindow.ui", NULL);
  gtk_builder_add_from_resource (builder, "/gui/MainWindow.ui", NULL);
  

  /* Connect signal handlers to the constructed widgets. */
  GObject *window = gtk_builder_get_object (builder, "window");
  gtk_window_set_application (GTK_WINDOW (window), app);

  GObject *button = gtk_builder_get_object (builder, "button1");
  g_signal_connect (button, "clicked", G_CALLBACK (print_hello), NULL);

  button = gtk_builder_get_object (builder, "button2");
  g_signal_connect (button, "clicked", G_CALLBACK (print_hello), NULL);

  button = gtk_builder_get_object (builder, "quit");
  g_signal_connect_swapped (button, "clicked", G_CALLBACK (quit_cb), window);

  gtk_widget_show (GTK_WIDGET (window));

  /* We do not need the builder any more */
  g_object_unref (builder);
}



