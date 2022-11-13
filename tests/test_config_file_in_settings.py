from pydantic import BaseSettings


class Settings(BaseSettings):
    config_filename: str = "config.ini"

    class Config:
        fields = {"config_filename": {"env": "config_file"}}


if __name__ == "__main__":
    settings = Settings()

    print(f"Config filename is: {settings.config_filename}")
