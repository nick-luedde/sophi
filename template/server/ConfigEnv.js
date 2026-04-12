class CurrentEnvironment {

  static config() {

    /**
     * LEAVE THIS COMMENT BELOW!
     */
    // {{# configurationTemplate }}

    return ConfigurationFactory.developmentConfig();
  }

}

class ConfigurationFactory {

  static developmentConfig() {
    return {
      environment: 'development',
      // ... other config values,
      secret: () => PropertiesService.getScriptProperties().getProperty('__dev-prop-name')
    };
  }

  static productionConfig() {
    return {
      environment: 'production',
      // ... other config values,
      secret: () => PropertiesService.getScriptProperties().getProperty('__prod-prop-name')
    };
  }

}