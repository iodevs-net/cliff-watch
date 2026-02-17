from django.db import models


class Category(models.Model):
    name = models.CharField(max_length=100, unique=True)
    description = models.TextField(blank=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        verbose_name_plural = "Categories"
        ordering = ["name"]

    def __str__(self):
        return self.name


class Prompt(models.Model):
    title = models.CharField(max_length=200)
    content = models.TextField()
    category = models.ForeignKey(
        Category, on_delete=models.SET_NULL, null=True, blank=True, related_name="prompts"
    )
    tags = models.CharField(max_length=500, blank=True, help_text="Comma-separated tags")
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
    is_favorite = models.BooleanField(default=False)

    class Meta:
        ordering = ["-created_at"]

    def __str__(self):
        return self.title

    def get_tags_list(self):
        return [tag.strip() for tag in self.tags.split(",") if tag.strip()]
